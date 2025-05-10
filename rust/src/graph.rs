// src/graph.rs
use crate::config::{LOGS_DIR_NAME, GRAPH_EXTENSION};
use plotters::prelude::*;
use plotters_bitmap::bitmap_pixel::RGBPixel;
use plotters_bitmap::BitMapBackend;
use std::path::PathBuf;

// Import specific style elements
use plotters::style::{RED, BLUE, WHITE, BLACK, Color};


pub const MATPLOTLIB_AVAILABLE: bool = true;

pub fn generate_balance_graph(
    balance_history: &[f64],
    run_timestamp: u64,
    starting_balance: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    if balance_history.len() < 2 {
        log::info!("Not enough data points to generate balance graph (need at least 2).");
        return Ok(());
    }

    let logs_dir = PathBuf::from(LOGS_DIR_NAME);
    let graph_filename = format!("{}.{}", run_timestamp, GRAPH_EXTENSION);
    let graph_file_path = logs_dir.join(&graph_filename);

    let root_area = BitMapBackend::<RGBPixel>::new(&graph_file_path, (1200, 600))
        .into_drawing_area();
    root_area.fill(&WHITE)?;

    let min_balance_val = balance_history.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_balance_val = balance_history.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    
    let y_range_height = (max_balance_val - min_balance_val).max(10.0);
    let y_padding = y_range_height * 0.1; 
    let y_min_plot = (min_balance_val - y_padding).min(starting_balance - y_padding);
    let y_max_plot = (max_balance_val + y_padding).max(starting_balance + y_padding);
    let y_min_final = if y_min_plot >= y_max_plot { y_max_plot - y_range_height.max(1.0) } else { y_min_plot };
    let y_max_final = if y_max_plot <= y_min_final { y_min_final + y_range_height.max(1.0) } else { y_max_plot };

    let mut chart = ChartBuilder::on(&root_area)
        .caption(format!("Player Balance Over Simulation (Run ID: {})", run_timestamp), ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..balance_history.len() -1 , y_min_final..y_max_final)?;

    chart.configure_mesh()
        .x_desc("Hand Number (0 = Initial State)")
        .y_desc("Player Balance ($)")
        .draw()?;

    // Plot balance history
    chart.draw_series(LineSeries::new(
        balance_history.iter().enumerate().map(|(i, &bal)| (i, bal)),
        BLUE.mix(0.8).stroke_width(2), 
    ))?
    .label("Balance")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.mix(0.8).filled()));

    // --- Fallback: Solid Red Line for Starting Balance ---
    let starting_balance_line_style = RED.mix(0.8).stroke_width(2); // Solid red line

    chart.draw_series(LineSeries::new(
        vec![(0, starting_balance), (balance_history.len() -1 , starting_balance)],
        starting_balance_line_style,
    ))?
    .label(format!("Starting Balance (${:.2})", starting_balance))
    .legend(|(x, y)| { 
        PathElement::new(vec![(x, y), (x + 20, y)], RED.mix(0.8).filled())
    });
    // --- END Fallback ---

    chart.configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root_area.present()?;
    log::info!("Balance graph saved to: {:?}", graph_file_path);
    println!("\nBalance graph saved to: {:?}", graph_file_path);

    Ok(())
}
