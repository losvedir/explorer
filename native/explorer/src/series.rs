use polars::prelude::*;
use rand::seq::IteratorRandom;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use rustler::{Encoder, Env, Term};
use std::result::Result;

use crate::{ExDataFrame, ExSeries, ExSeriesRef, ExplorerError};

pub(crate) fn to_series_collection(s: Vec<ExSeries>) -> Vec<Series> {
    s.into_iter().map(|c| c.resource.0.clone()).collect()
}

pub(crate) fn to_ex_series_collection(s: Vec<Series>) -> Vec<ExSeries> {
    s.into_iter().map(|c| ExSeries::new(c)).collect()
}

#[rustler::nif]
pub fn s_as_str(data: ExSeries) -> Result<String, ExplorerError> {
    Ok(format!("{:?}", data.resource.0))
}

macro_rules! init_method {
    ($name:ident, $type:ty) => {
        #[rustler::nif]
        pub fn $name(name: &str, val: Vec<Option<$type>>) -> ExSeries {
            ExSeries::new(Series::new(name, val.as_slice()))
        }
    };
    ($name:ident, $type:ty, $cast_type:ty) => {
        #[rustler::nif]
        pub fn $name(name: &str, val: Vec<Option<$type>>) -> ExSeries {
            ExSeries::new(
                Series::new(name, val.as_slice())
                    .cast::<$cast_type>()
                    .unwrap(),
            )
        }
    };
}

init_method!(s_new_i64, i64);
init_method!(s_new_bool, bool);
init_method!(s_new_date32, &str, Date32Type);
init_method!(s_new_date64, &str, Date64Type);
init_method!(s_new_f64, f64);
init_method!(s_new_str, String);

#[rustler::nif]
pub fn s_rechunk(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let series = s.rechunk();
    Ok(ExSeries::new(series))
}

#[rustler::nif]
pub fn s_name(data: ExSeries) -> Result<String, ExplorerError> {
    Ok(data.resource.0.name().to_string())
}

#[rustler::nif]
pub fn s_rename(data: ExSeries, name: &str) -> Result<ExSeries, ExplorerError> {
    let mut s = data.resource.0.clone();
    s.rename(name);
    Ok(ExSeries::new(s))
}

#[rustler::nif]
pub fn s_dtype(data: ExSeries) -> Result<String, ExplorerError> {
    let s = &data.resource.0;
    let dt = s.dtype().to_string();
    Ok(dt)
}

#[rustler::nif]
pub fn s_n_chunks(data: ExSeries) -> Result<usize, ExplorerError> {
    let s = &data.resource.0;
    Ok(s.n_chunks())
}

#[rustler::nif]
pub fn s_limit(data: ExSeries, num_elements: usize) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let series = s.limit(num_elements);
    Ok(ExSeries::new(series))
}

#[rustler::nif]
pub fn s_slice(data: ExSeries, offset: i64, length: usize) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let series = s.slice(offset, length);
    Ok(ExSeries::new(series))
}

#[rustler::nif]
pub fn s_append(data: ExSeries, other: ExSeries) -> Result<ExSeries, ExplorerError> {
    let mut s = data.resource.0.clone();
    let s1 = &other.resource.0;
    s.append(s1)?;
    Ok(ExSeries::new(s))
}

#[rustler::nif]
pub fn s_filter(data: ExSeries, filter: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &filter.resource.0;
    if let Ok(ca) = s1.bool() {
        let series = s.filter(ca)?;
        Ok(ExSeries::new(series))
    } else {
        Err(ExplorerError::Other("Expected a boolean mask".into()))
    }
}

#[rustler::nif]
pub fn s_add(data: ExSeries, other: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &other.resource.0;
    Ok(ExSeries::new(s + s1))
}

#[rustler::nif]
pub fn s_sub(data: ExSeries, other: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &other.resource.0;
    Ok(ExSeries::new(s - s1))
}

#[rustler::nif]
pub fn s_mul(data: ExSeries, other: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &other.resource.0;
    Ok(ExSeries::new(s * s1))
}

#[rustler::nif]
pub fn s_div(data: ExSeries, other: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &other.resource.0;
    Ok(ExSeries::new(s / s1))
}

#[rustler::nif]
pub fn s_head(data: ExSeries, length: Option<usize>) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.head(length)))
}

#[rustler::nif]
pub fn s_tail(data: ExSeries, length: Option<usize>) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.tail(length)))
}

#[rustler::nif]
pub fn s_sort(data: ExSeries, reverse: bool) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.sort(reverse)))
}

#[rustler::nif]
pub fn s_argsort(data: ExSeries, reverse: bool) -> Result<Vec<Option<u32>>, ExplorerError> {
    let s = &data.resource.0;
    Ok(s.argsort(reverse).into_iter().collect::<Vec<Option<u32>>>())
}

#[rustler::nif]
pub fn s_unique(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let unique = s.unique()?;
    Ok(ExSeries::new(unique))
}

#[rustler::nif]
pub fn s_value_counts(data: ExSeries) -> Result<ExDataFrame, ExplorerError> {
    let s = &data.resource.0;
    let mut df = s.value_counts()?;
    let df = df
        .may_apply("counts", |s: &Series| s.cast::<Int64Type>())?
        .clone();
    Ok(ExDataFrame::new(df))
}

#[rustler::nif]
pub fn s_take(data: ExSeries, indices: Vec<u32>) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let idx = UInt32Chunked::new_from_slice("idx", indices.as_slice());
    let s1 = s.take(&idx)?;
    Ok(ExSeries::new(s1))
}

#[rustler::nif]
pub fn s_null_count(data: ExSeries) -> Result<usize, ExplorerError> {
    let s = &data.resource.0;
    Ok(s.null_count())
}

#[rustler::nif]
pub fn s_is_null(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.is_null().into_series()))
}

#[rustler::nif]
pub fn s_is_not_null(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.is_not_null().into_series()))
}

#[rustler::nif]
pub fn s_is_unique(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let ca = s.is_unique()?;
    Ok(ExSeries::new(ca.into_series()))
}

#[rustler::nif]
pub fn s_arg_true(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let ca = s.arg_true()?;
    Ok(ExSeries::new(ca.into_series()))
}

#[rustler::nif]
pub fn s_is_duplicated(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let ca = s.is_duplicated()?;
    Ok(ExSeries::new(ca.into_series()))
}

#[rustler::nif]
pub fn s_explode(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = s.explode()?;
    Ok(ExSeries::new(s1))
}

#[rustler::nif]
pub fn s_take_every(data: ExSeries, n: usize) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = s.take_every(n);
    Ok(ExSeries::new(s1))
}

#[rustler::nif]
pub fn s_series_equal(
    data: ExSeries,
    other: ExSeries,
    null_equal: bool,
) -> Result<bool, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &other.resource.0;
    let result = if null_equal {
        s.series_equal_missing(s1)
    } else {
        s.series_equal(s1)
    };
    Ok(result)
}

#[rustler::nif]
pub fn s_eq(data: ExSeries, rhs: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &rhs.resource.0;
    Ok(ExSeries::new(s.eq(s1).into_series()))
}

#[rustler::nif]
pub fn s_neq(data: ExSeries, rhs: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &rhs.resource.0;
    Ok(ExSeries::new(s.neq(s1).into_series()))
}

#[rustler::nif]
pub fn s_gt(data: ExSeries, rhs: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &rhs.resource.0;
    Ok(ExSeries::new(s.gt(s1).into_series()))
}

#[rustler::nif]
pub fn s_gt_eq(data: ExSeries, rhs: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &rhs.resource.0;
    Ok(ExSeries::new(s.gt_eq(s1).into_series()))
}

#[rustler::nif]
pub fn s_lt(data: ExSeries, rhs: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &rhs.resource.0;
    Ok(ExSeries::new(s.lt(s1).into_series()))
}

#[rustler::nif]
pub fn s_lt_eq(data: ExSeries, rhs: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = &rhs.resource.0;
    Ok(ExSeries::new(s.lt_eq(s1).into_series()))
}

#[rustler::nif]
pub fn s_not(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let bool = s.bool()?;
    Ok(ExSeries::new((!bool).into_series()))
}

#[rustler::nif]
pub fn s_len(data: ExSeries) -> Result<usize, ExplorerError> {
    let s = &data.resource.0;
    Ok(s.len())
}

#[rustler::nif]
pub fn s_drop_nulls(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.drop_nulls()))
}

#[rustler::nif]
pub fn s_fill_none(data: ExSeries, strategy: &str) -> Result<ExSeries, ExplorerError> {
    let strat = match strategy {
        "backward" => FillNullStrategy::Backward,
        "forward" => FillNullStrategy::Forward,
        "min" => FillNullStrategy::Min,
        "max" => FillNullStrategy::Max,
        "mean" => FillNullStrategy::Mean,
        s => return Err(ExplorerError::Other(format!("Strategy {} not supported", s)).into()),
    };

    let s = &data.resource.0;
    let s1 = s.fill_null(strat)?;
    Ok(ExSeries::new(s1))
}

#[rustler::nif]
pub fn s_clone(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.clone()))
}

#[rustler::nif]
pub fn s_shift(data: ExSeries, periods: i64) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let s1 = s.shift(periods);
    Ok(ExSeries::new(s1))
}

#[rustler::nif]
pub fn s_zip_with(
    data: ExSeries,
    mask: ExSeries,
    other: ExSeries,
) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let m = &mask.resource.0;
    let s1 = &other.resource.0;
    let msk = m.bool()?;
    let s2 = s.zip_with(msk, s1)?;
    Ok(ExSeries::new(s2))
}

#[rustler::nif]
pub fn s_str_lengths(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let ca = s.utf8()?;
    let s1 = ca.str_lengths().into_series();
    Ok(ExSeries::new(s1))
}

#[rustler::nif]
pub fn s_str_contains(data: ExSeries, pat: &str) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let ca = s.utf8()?;
    let s1 = ca.contains(pat)?.into_series();
    Ok(ExSeries::new(s1))
}

#[rustler::nif]
pub fn s_str_replace(data: ExSeries, pat: &str, val: &str) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let ca = s.utf8()?;
    let s1 = ca.replace(pat, val)?.into_series();
    Ok(ExSeries::new(s1))
}

#[rustler::nif]
pub fn s_str_replace_all(data: ExSeries, pat: &str, val: &str) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let ca = s.utf8()?;
    let s1 = ca.replace_all(pat, val)?.into_series();
    Ok(ExSeries::new(s1))
}

#[rustler::nif]
pub fn s_str_to_uppercase(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let ca = s.utf8()?;
    let s1 = ca.to_uppercase().into_series();
    Ok(ExSeries::new(s1))
}

#[rustler::nif]
pub fn s_str_to_lowercase(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let ca = s.utf8()?;
    let s1 = ca.to_lowercase().into_series();
    Ok(ExSeries::new(s1))
}

#[rustler::nif]
pub fn s_str_parse_date32(data: ExSeries, fmt: Option<&str>) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    if let Ok(ca) = s.utf8() {
        let ca = ca.as_date32(fmt)?;
        Ok(ExSeries::new(ca.into_series()))
    } else {
        Err(ExplorerError::Other("cannot parse date32 expected utf8 type".into()).into())
    }
}

#[rustler::nif]
pub fn s_str_parse_date64(data: ExSeries, fmt: Option<&str>) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    if let Ok(ca) = s.utf8() {
        let ca = ca.as_date64(fmt)?;
        Ok(ExSeries::new(ca.into_series()))
    } else {
        Err(ExplorerError::Other("cannot parse date64 expected utf8 type".into()).into())
    }
}

#[rustler::nif]
pub fn s_to_dummies(data: ExSeries) -> Result<ExDataFrame, ExplorerError> {
    let s = &data.resource.0;
    let df = s.to_dummies()?;
    Ok(ExDataFrame::new(df))
}

#[rustler::nif]
pub fn s_rolling_sum(
    data: ExSeries,
    window_size: u32,
    weight: Option<Vec<f64>>,
    ignore_null: bool,
    min_periods: Option<u32>,
) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    let min_periods = if let Some(mp) = min_periods {
        mp
    } else {
        window_size
    };
    let s1 = s.rolling_sum(window_size, weight.as_deref(), ignore_null, min_periods)?;
    Ok(ExSeries::new(s1))
}

#[rustler::nif]
pub fn s_rolling_mean(
    data: ExSeries,
    window_size: u32,
    weight: Option<Vec<f64>>,
    ignore_null: bool,
    min_periods: Option<u32>,
) -> Result<ExSeries, ExplorerError> {
    let min_periods = if let Some(mp) = min_periods {
        mp
    } else {
        window_size
    };
    let s = &data.resource.0;
    let s1 = s.rolling_mean(window_size, weight.as_deref(), ignore_null, min_periods)?;
    Ok(ExSeries::new(s1))
}

#[rustler::nif]
pub fn s_rolling_max(
    data: ExSeries,
    window_size: u32,
    weight: Option<Vec<f64>>,
    ignore_null: bool,
    min_periods: Option<u32>,
) -> Result<ExSeries, ExplorerError> {
    let min_periods = if let Some(mp) = min_periods {
        mp
    } else {
        window_size
    };
    let s = &data.resource.0;
    let s1 = s.rolling_max(window_size, weight.as_deref(), ignore_null, min_periods)?;
    Ok(ExSeries::new(s1))
}

#[rustler::nif]
pub fn s_rolling_min(
    data: ExSeries,
    window_size: u32,
    weight: Option<Vec<f64>>,
    ignore_null: bool,
    min_periods: Option<u32>,
) -> Result<ExSeries, ExplorerError> {
    let min_periods = if let Some(mp) = min_periods {
        mp
    } else {
        window_size
    };
    let s = &data.resource.0;
    let s1 = s.rolling_min(window_size, weight.as_deref(), ignore_null, min_periods)?;
    Ok(ExSeries::new(s1))
}

#[rustler::nif]
pub fn s_to_list(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    let s = ExSeriesRef(data.resource.0.clone());
    Ok(s.encode(env))
}

#[rustler::nif]
pub fn s_sum(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    let s = &data.resource.0;
    match s.dtype() {
        DataType::Boolean => Ok(s.sum::<i64>().encode(env)),
        DataType::Int64 => Ok(s.sum::<i64>().encode(env)),
        DataType::Float64 => Ok(s.sum::<f64>().encode(env)),
        dt => panic!("sum/1 not implemented for {:?}", dt),
    }
}

#[rustler::nif]
pub fn s_min(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    let s = &data.resource.0;
    match s.dtype() {
        DataType::Int64 => Ok(s.min::<i64>().encode(env)),
        DataType::Float64 => Ok(s.min::<f64>().encode(env)),
        DataType::Date32 => Ok(s.min::<i32>().encode(env)),
        DataType::Date64 => Ok(s.min::<i64>().encode(env)),
        dt => panic!("min/1 not implemented for {:?}", dt),
    }
}

#[rustler::nif]
pub fn s_max(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    let s = &data.resource.0;
    match s.dtype() {
        DataType::Int64 => Ok(s.max::<i64>().encode(env)),
        DataType::Float64 => Ok(s.max::<f64>().encode(env)),
        DataType::Date32 => Ok(s.max::<i32>().encode(env)),
        DataType::Date64 => Ok(s.max::<i64>().encode(env)),
        dt => panic!("max/1 not implemented for {:?}", dt),
    }
}

#[rustler::nif]
pub fn s_mean(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    let s = &data.resource.0;
    match s.dtype() {
        DataType::Boolean => Ok(s.mean().encode(env)),
        DataType::Int64 => Ok(s.mean().encode(env)),
        DataType::Float64 => Ok(s.mean().encode(env)),
        dt => panic!("mean/1 not implemented for {:?}", dt),
    }
}

#[rustler::nif]
pub fn s_median(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    let s = &data.resource.0;
    match s.dtype() {
        DataType::Int64 => Ok(s.median().encode(env)),
        DataType::Float64 => Ok(s.median().encode(env)),
        dt => panic!("median/1 not implemented for {:?}", dt),
    }
}

#[rustler::nif]
pub fn s_var(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    let s = &data.resource.0;
    match s.dtype() {
        DataType::Int64 => Ok(s.i64().unwrap().var().encode(env)),
        DataType::Float64 => Ok(s.f64().unwrap().var().encode(env)),
        dt => panic!("var/1 not implemented for {:?}", dt),
    }
}

#[rustler::nif]
pub fn s_std(env: Env, data: ExSeries) -> Result<Term, ExplorerError> {
    let s = &data.resource.0;
    match s.dtype() {
        DataType::Int64 => Ok(s.i64().unwrap().std().encode(env)),
        DataType::Float64 => Ok(s.f64().unwrap().std().encode(env)),
        dt => panic!("std/1 not implemented for {:?}", dt),
    }
}

#[rustler::nif]
pub fn s_get(env: Env, data: ExSeries, idx: usize) -> Result<Term, ExplorerError> {
    let s = &data.resource.0;
    let term: Term = match s.get(idx) {
        AnyValue::Null => None::<bool>.encode(env),
        AnyValue::Boolean(v) => Some(v).encode(env),
        AnyValue::Utf8(v) => Some(v).encode(env),
        AnyValue::Int64(v) => Some(v).encode(env),
        AnyValue::Float64(v) => Some(v).encode(env),
        AnyValue::Date32(v) => Some(v).encode(env),
        AnyValue::Date64(v) => Some(v).encode(env),
        dt => panic!("get/2 not implemented for {:?}", dt),
    };
    Ok(term)
}

#[rustler::nif]
pub fn s_cum_sum(data: ExSeries, reverse: bool) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.cumsum(reverse)))
}

#[rustler::nif]
pub fn s_cum_max(data: ExSeries, reverse: bool) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.cummax(reverse)))
}

#[rustler::nif]
pub fn s_cum_min(data: ExSeries, reverse: bool) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.cummin(reverse)))
}

#[rustler::nif]
pub fn s_quantile(data: ExSeries, quantile: f64) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.quantile_as_series(quantile)?))
}

#[rustler::nif]
pub fn s_peak_max(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.peak_max().into_series()))
}

#[rustler::nif]
pub fn s_peak_min(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.peak_min().into_series()))
}

#[rustler::nif]
pub fn s_reverse(data: ExSeries) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.reverse()))
}

#[rustler::nif]
pub fn s_n_unique(data: ExSeries) -> Result<usize, ExplorerError> {
    let s = &data.resource.0;
    Ok(s.n_unique()?)
}

#[rustler::nif]
pub fn s_pow(data: ExSeries, exponent: f64) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(s.pow(exponent)?))
}

#[rustler::nif]
pub fn s_cast(data: ExSeries, to_type: &str) -> Result<ExSeries, ExplorerError> {
    let s = &data.resource.0;
    Ok(ExSeries::new(cast(s, to_type)?))
}

pub fn cast(s: &Series, to_type: &str) -> Result<Series, PolarsError> {
    match to_type {
        "float" => Ok(s.cast::<Float64Type>()?),
        "integer" => Ok(s.cast::<Int64Type>()?),
        "date" => Ok(s.cast::<Date32Type>()?),
        "datetime" => Ok(s.cast::<Date64Type>()?),
        "boolean" => Ok(s.cast::<BooleanType>()?),
        "string" => Ok(s.cast::<Utf8Type>()?),
        _ => panic!("Cannot cast to type"),
    }
}

#[rustler::nif]
pub fn s_seedable_random_indices(
    length: usize,
    n_samples: usize,
    with_replacement: bool,
    seed: u64,
) -> Vec<usize> {
    let mut rng: Pcg64 = SeedableRng::seed_from_u64(seed);
    let range: Vec<usize> = (0..length).collect();
    if with_replacement {
        (0..n_samples).map(|_| rng.gen_range(0..length)).collect()
    } else {
        range
            .iter()
            .choose_multiple(&mut rng, n_samples)
            .iter()
            .map(|x| **x)
            .collect()
    }
}
