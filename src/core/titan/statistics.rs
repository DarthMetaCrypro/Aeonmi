//! Statistical functions for data analysis.

use std::collections::HashMap;

/// Calculates the mean (average) of a dataset.
pub fn mean(data: &[f64]) -> Result<f64, &'static str> {
    if data.is_empty() {
        Err("Data array cannot be empty.")
    } else {
        Ok(data.iter().sum::<f64>() / data.len() as f64)
    }
}

/// Calculates the variance of a dataset.
pub fn variance(data: &[f64]) -> Result<f64, &'static str> {
    if data.is_empty() {
        Err("Data array cannot be empty.")
    } else {
        let mean_value = mean(data)?;
        Ok(data.iter().map(|x| (x - mean_value).powi(2)).sum::<f64>() / data.len() as f64)
    }
}

/// Calculates the standard deviation of a dataset.
pub fn standard_deviation(data: &[f64]) -> Result<f64, &'static str> {
    variance(data).map(|v| v.sqrt())
}

/// Calculates the median of a dataset.
pub fn median(data: &mut [f64]) -> Result<f64, &'static str> {
    if data.is_empty() {
        Err("Data array cannot be empty.")
    } else {
        data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = data.len();
        if n % 2 == 0 {
            Ok((data[n / 2 - 1] + data[n / 2]) / 2.0f64)
        } else {
            Ok(data[n / 2])
        }
    }
}

/// Calculates the mode(s) of a dataset.
pub fn mode(data: &[f64]) -> Result<Vec<f64>, &'static str> {
    if data.is_empty() {
        Err("Data array cannot be empty.")
    } else {
        // Scale/round to bucket floating-point values for equality.
        let mut frequency_map: HashMap<i64, usize> = HashMap::new();
        for &value in data {
            let key = (value * 1e6f64).round() as i64;
            *frequency_map.entry(key).or_insert(0) += 1;
        }
        let max_frequency = frequency_map.values().copied().max().unwrap_or(0);
        let modes = frequency_map
            .into_iter()
            .filter(|&(_, count)| count == max_frequency)
            .map(|(key, _)| key as f64 / 1e6f64)
            .collect();
        Ok(modes)
    }
}

/// Calculates the p-th percentile of a dataset (linear interpolation).
pub fn percentile(data: &mut [f64], p: f64) -> Result<f64, &'static str> {
    if data.is_empty() {
        Err("Data array cannot be empty.")
    } else if !(0.0f64..=100.0f64).contains(&p) {
        Err("Percentile must be between 0 and 100.")
    } else {
        data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let rank = (p / 100.0f64) * (data.len() as f64 - 1.0f64);
        let lower_index = rank.floor() as usize;
        let upper_index = rank.ceil() as usize;
        if lower_index == upper_index {
            Ok(data[lower_index])
        } else {
            let f = rank - lower_index as f64; // fractional part
            Ok((1.0f64 - f) * data[lower_index] + f * data[upper_index])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        assert_eq!(mean(&[1.0, 2.0, 3.0]).unwrap(), 2.0);
    }

    #[test]
    fn test_variance() {
        assert_eq!(variance(&[1.0, 2.0, 3.0]).unwrap(), 2.0f64 / 3.0f64);
    }

    #[test]
    fn test_standard_deviation() {
        assert_eq!(
            standard_deviation(&[1.0, 2.0, 3.0]).unwrap(),
            (2.0f64 / 3.0f64).sqrt()
        );
    }

    #[test]
    fn test_median() {
        let mut data = vec![3.0, 1.0, 2.0];
        assert_eq!(median(&mut data).unwrap(), 2.0);
    }

    #[test]
    fn test_mode() {
        assert_eq!(mode(&[1.0, 2.0, 2.0, 3.0]).unwrap(), vec![2.0]);
    }

    #[test]
    fn test_percentile() {
        let mut data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&mut data, 50.0).unwrap(), 3.0);
    }
}
