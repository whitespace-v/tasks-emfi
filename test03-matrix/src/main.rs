#![warn(clippy::all, clippy::pedantic)]

use std::sync::{Arc, Mutex};
use std::thread;

fn matrix_multiply(matrix_a: &[Vec<i32>], matrix_b: &[Vec<i32>]) -> Vec<Vec<i32>> {
    if matrix_a.is_empty() || matrix_b.is_empty() {
        return Vec::new();
    }
    let rows_a = matrix_a.len();
    let cols_a = matrix_a[0].len();
    let cols_b = matrix_b[0].len();

    let result = Arc::new(Mutex::new(vec![vec![0; cols_b]; rows_a]));

    let mut handles = vec![];
    for i in 0..rows_a {
        let matrix_a = matrix_a.to_vec();
        let matrix_b = matrix_b.to_vec();
        let result = Arc::clone(&result);

        let handle = thread::spawn(move || {
            for j in 0..cols_b {
                let mut sum = 0;
                for k in 0..cols_a {
                    sum += matrix_a[i][k] * matrix_b[k][j];
                }
                let mut result = result.lock().unwrap();
                result[i][j] = sum;
            }
        });

        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    Arc::try_unwrap(result).unwrap().into_inner().unwrap()
}

fn main() {
    let a = vec![vec![1, 3], vec![4, 6]];
    let b = vec![vec![7, 8, 9], vec![10, 11, 12], vec![13, 14, 15]];

    println!("{:?}", matrix_multiply(&a, &b));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiplication() {
        let a = vec![vec![1, 2], vec![3, 4]];
        let b = vec![vec![5, 6], vec![7, 8]];

        assert_eq!(matrix_multiply(&a, &b), vec![vec![19, 22], vec![43, 50]]);
    }

    #[test]
    fn test_zero_matrix_multiplication() {
        let a = vec![vec![1, 2], vec![3, 4]];
        let b = vec![vec![0, 0], vec![0, 0]];

        assert_eq!(matrix_multiply(&a, &b), vec![vec![0, 0], vec![0, 0]]);
    }

    #[test]
    fn test_different_sizes_multiplication() {
        let a = vec![vec![1, 3], vec![4, 6]];
        let b = vec![vec![7, 8, 9], vec![10, 11, 12], vec![13, 14, 15]];

        assert_eq!(
            matrix_multiply(&a, &b),
            vec![vec![37, 41, 45], vec![88, 98, 108]]
        );
    }

    #[test]
    fn test_empty_matrices() {
        let a: Vec<Vec<i32>> = vec![];
        let b: Vec<Vec<i32>> = vec![];

        let expected: Vec<Vec<i32>> = vec![];
        assert_eq!(matrix_multiply(&a, &b), expected);
    }
}
