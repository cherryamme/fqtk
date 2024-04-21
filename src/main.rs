use std::collections::HashMap;
use std::sync::Mutex;
use bio::io::fastq;
use serde_json::json;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use std::io;
use clap::Parser;
use std::fs::File;
use plotters::prelude::*;

/// Simple program to greet a person
#[derive(Parser, Debug)]

#[command(version, author, about, long_about = None)]
struct Args {
    /// The name of output file
    #[arg(short, long, default_value = "out.json")]
    output: String,
    /// Number of threads
    #[arg(short, long, default_value = "4")]
    threads: usize,
    /// whether to plot the data
    #[arg(short, long, default_value = "false")]
    plot: bool,
}


fn plotter(data: &HashMap<usize, usize>) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("plot.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Plot from HashMap", ("Arial", 20).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0usize..200usize, 0usize..7000usize)?;

    chart.configure_mesh().draw()?;

    let max_key = *data.keys().max().unwrap_or(&0);
    let mut data_vec: Vec<(usize, usize)> = (0..=max_key)
        .map(|key| (key, *data.get(&key).unwrap_or(&0)))
        .collect();
    data_vec.sort_by(|a, b| a.0.cmp(&b.0));

    chart.draw_series(LineSeries::new(data_vec, &RED))?;

    root.present()?;
    Ok(())
}


fn main() {
    let args = Args::parse();
    ThreadPoolBuilder::new().num_threads(args.threads).build_global().unwrap();
    let stdin: io::Stdin = io::stdin();
    let reader = fastq::Reader::new(stdin.lock());


    let records: Vec<_> = reader.records().collect::<Result<_, _>>().unwrap();

    let lengths: Mutex<HashMap<usize, usize>> = Mutex::new(HashMap::new());

    records.par_iter().for_each(|record: &fastq::Record| {
        let record: &fastq::Record = record;
        let length: usize = record.seq().len();
        let mut lengths: std::sync::MutexGuard<'_, HashMap<usize, usize>> = lengths.lock().unwrap();
        *lengths.entry(length).or_insert(0) += 1;
    });

    let output = lengths.into_inner().unwrap();
    let output_file = File::create(&args.output).unwrap();
    if args.plot {
    plotter(&output).unwrap();
    }
    serde_json::to_writer(output_file, &json!(output)).unwrap();
}
