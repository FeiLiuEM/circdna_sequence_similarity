use csv::Writer;
use std::env;
use std::fs::File;
use std::process;

// Smith-Waterman算法的实现
fn smith_waterman(a: &str, b: &str) -> i32 {
    let m = a.len();
    let n = b.len();
    let mut matrix = vec![vec![0; n + 1]; m + 1];
    let match_score = 2;
    let mismatch_score = -2;
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
    if args.len() != 3 {
        eprintln!("Usage: {} <a_string> <b_string>", args[0]);
        process::exit(1);
    }

    let a_string = &args[1];
    let b_string = &args[2];

    if b_string.len() != 20 {
        eprintln!("b_string must be exactly 20 characters long.");
        process::exit(1);
    }

    let mut similarity_list = Vec::new();

    for i in (0..a_string.len()).step_by(10) {
        let end = (i + 30).min(a_string.len());
        if end - i < 30 {
            break;
        }
        let sub_string = &a_string[i..end];
        let similarity = smith_waterman(sub_string, b_string);
        similarity_list.push(similarity);
    }

    let similarity_string = similarity_list.iter().map(|&x| x.to_string()).collect::<Vec<_>>().join(",");

    let file = File::create("result.csv").expect("Unable to create file");
    let mut wtr = Writer::from_writer(file);
    wtr.write_record(&["a_string", "b_string", "similarity_string"]).expect("Unable to write record");
    wtr.write_record(&[a_string, b_string, &similarity_string]).expect("Unable to write record");
    wtr.flush().expect("Unable to flush writer");
}

