use csv::Reader;
use csv::Writer;
use std::fs::File;
//use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::env;

// Smith-Waterman算法的实现
fn smith_waterman(a: &str, b: &str) -> i32 {
    let m = a.len();
    let n = b.len();
    let mut matrix = vec![vec![0; n + 1]; m + 1];
    let match_score = 2;
    let mismatch_score = -1;
    let gap_opening_penalty = -1;
    let gap_extension_penalty = -1;

    for i in 1..=m {
        for j in 1..=n {
            let match_value = if a.chars().nth(i - 1).unwrap() == b.chars().nth(j - 1).unwrap() {
                match_score
            } else {
                mismatch_score
            };

            let mut gap_penalty_horizontal = gap_opening_penalty;
            let mut gap_penalty_vertical = gap_opening_penalty;

            for k in (1..i).rev() {
                if matrix[k][j] != 0 {
                    gap_penalty_vertical = gap_extension_penalty;
                    break;
                }
            }

            for k in (1..j).rev() {
                if matrix[i][k] != 0 {
                    gap_penalty_horizontal = gap_extension_penalty;
                    break;
                }
            }

            matrix[i][j] = *[
                matrix[i - 1][j - 1] + match_value,
                matrix[i - 1][j] + gap_penalty_vertical,
                matrix[i][j - 1] + gap_penalty_horizontal,
                0,
            ]
            .iter()
            .max()
            .unwrap();
        }
    }

    let mut max_similarity = 0;
    for i in 0..=m {
        for j in 0..=n {
            if matrix[i][j] > max_similarity {
                max_similarity = matrix[i][j];
            }
        }
    }

    max_similarity
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut num_threads = 8;

    if let Some(index) = args.iter().position(|arg| arg == "-j") {
        if index + 1 < args.len() {
            num_threads = args[index + 1].parse::<usize>().expect("Invalid number of threads");
        }
    }

    let a_file = File::open("a_sequence.csv").expect("Unable to open a_sequence.csv");
    let mut a_rdr = Reader::from_reader(a_file);

    let b_file = File::open("b_sequence.csv").expect("Unable to open b_sequence.csv");
    let mut b_rdr = Reader::from_reader(b_file);

    let mut a_sequences = Vec::new();
    for result in a_rdr.records() {
        let record = result.expect("Unable to read record");
        let a_sequence = record.get(0).expect("Unable to get a_sequence").to_uppercase();
        a_sequences.push(a_sequence);
    }

    let mut b_sequences = Vec::new();
    for result in b_rdr.records() {
        let record = result.expect("Unable to read record");
        let b_sequence = record.get(0).expect("Unable to get b_sequence").to_uppercase();
        //if b_sequence.len() != 20 {
        //    eprintln!("b_sequence must be exactly 20 characters long.");
        //    process::exit(1);
        //}
        b_sequences.push(b_sequence);
    }

    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    // 分配任务给多个线程
    let chunk_size = (a_sequences.len() + num_threads - 1) / num_threads;
    for chunk in a_sequences.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let results = Arc::clone(&results);
        let b_sequences = b_sequences.clone();

        let handle = thread::spawn(move || {
            for a_sequence in chunk {
                for b_sequence in &b_sequences {
                    // 预处理a_sequence为a_sequence + a_sequence[0:20]
                    let extended_a_sequence = format!("{}{}", a_sequence, &a_sequence[0..20.min(a_sequence.len())]);

                    let mut similarity_list = Vec::new();

                    for i in (0..extended_a_sequence.len()).step_by(10) {
                        let end = (i + 30).min(extended_a_sequence.len());
                        if end - i < 30 {
                            break;
                        }
                        let sub_sequence = &extended_a_sequence[i..end];
                        let similarity = smith_waterman(sub_sequence, b_sequence);
                        similarity_list.push(similarity);
                    }

                    let similarity_string = similarity_list.iter().map(|&x| x.to_string()).collect::<Vec<_>>().join(",");

                    let mut results = results.lock().unwrap();
                    results.push(vec![a_sequence.clone(), b_sequence.clone(), similarity_string]);
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let results = results.lock().unwrap();
    let mut wtr = Writer::from_path("result.csv").expect("Unable to create file");
    wtr.write_record(&["a_sequence", "b_sequence", "similarity_string"]).expect("Unable to write record");
    for result in results.iter() {
        wtr.write_record(result).expect("Unable to write record");
    }

    wtr.flush().expect("Unable to flush writer");
}

