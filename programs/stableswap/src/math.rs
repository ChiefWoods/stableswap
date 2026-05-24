use anchor_lang::prelude::*;

use crate::{error::StableSwapError, MAX_FEE_BPS};

pub fn compute_d(reserves: &[u128], amp: u128) -> Result<u128> {
    let n = reserves.len() as u128;
    if n == 0 {
        return Ok(0);
    }

    let sum: u128 = reserves.iter().sum();
    if sum == 0 {
        return Ok(0);
    }

    // A * n^n
    let ann = amp
        .checked_mul(n.pow(n as u32))
        .ok_or(StableSwapError::MathOverflow)?;

    let mut d = sum;

    for _ in 0..255 {
        let d_prev = d;

        // D_P = D^(n+1) / (n^n * prod(reserves))
        let mut d_p = d;
        for &reserve in reserves {
            if reserve == 0 {
                return Err(StableSwapError::EmptyPool.into());
            }
            d_p = d_p
                .checked_mul(d)
                .ok_or(StableSwapError::MathOverflow)?
                .checked_div(
                    reserve
                        .checked_mul(n)
                        .ok_or(StableSwapError::MathOverflow)?,
                )
                .ok_or(StableSwapError::MathOverflow)?;
        }

        // Newton's method:
        // d = (ann * sum + d_p * n) * d / ((ann - 1) * d + (n + 1) * d_p)
        let numerator = ann
            .checked_mul(sum)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_add(d_p.checked_mul(n).ok_or(StableSwapError::MathOverflow)?)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_mul(d)
            .ok_or(StableSwapError::MathOverflow)?;

        let denominator = ann
            .checked_sub(1)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_mul(d)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_add(
                n.checked_add(1)
                    .ok_or(StableSwapError::MathOverflow)?
                    .checked_mul(d_p)
                    .ok_or(StableSwapError::MathOverflow)?,
            )
            .ok_or(StableSwapError::MathOverflow)?;

        d = numerator
            .checked_div(denominator)
            .ok_or(StableSwapError::MathOverflow)?;

        // Convergence check
        if d.abs_diff(d_prev) <= 1 {
            return Ok(d);
        }
    }

    Err(StableSwapError::ConvergenceFailed.into())
}

pub fn compute_y(
    reserves: &[u128],
    i: usize, // input token index
    j: usize, // output token index
    new_reserve_i: u128,
    amp: u128,
) -> Result<u128> {
    let n = reserves.len() as u128;
    let d = compute_d(reserves, amp)?;
    let ann = amp
        .checked_mul(n.pow(n as u32))
        .ok_or(StableSwapError::MathOverflow)?;

    // c = D^(n+1) / (n^n * prod(reserves_except_j) * ann)
    let mut c = d;
    let mut s: u128 = 0;

    for (k, &reserve) in reserves.iter().enumerate() {
        let x = if k == i {
            new_reserve_i
        } else if k == j {
            continue; // Skip j, we're solving for it
        } else {
            reserve
        };

        s = s.checked_add(x).ok_or(StableSwapError::MathOverflow)?;
        c = c
            .checked_mul(d)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_div(x.checked_mul(n).ok_or(StableSwapError::MathOverflow)?)
            .ok_or(StableSwapError::MathOverflow)?;
    }

    c = c
        .checked_mul(d)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_div(ann.checked_mul(n).ok_or(StableSwapError::MathOverflow)?)
        .ok_or(StableSwapError::MathOverflow)?;

    let b = s
        .checked_add(d.checked_div(ann).ok_or(StableSwapError::MathOverflow)?)
        .ok_or(StableSwapError::MathOverflow)?;

    // Newton's method to solve for y
    let mut y = d;

    for _ in 0..255 {
        let y_prev = y;

        // y = (y^2 + c) / (2y + b - d)
        let numerator = y
            .checked_mul(y)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_add(c)
            .ok_or(StableSwapError::MathOverflow)?;

        let denominator = y
            .checked_mul(2)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_add(b)
            .ok_or(StableSwapError::MathOverflow)?
            .checked_sub(d)
            .ok_or(StableSwapError::MathOverflow)?;

        y = numerator
            .checked_div(denominator)
            .ok_or(StableSwapError::MathOverflow)?;

        if y.abs_diff(y_prev) <= 1 {
            return Ok(y);
        }
    }

    Err(StableSwapError::ConvergenceFailed.into())
}

pub fn calculate_swap(
    reserves: &[u128],
    input_index: usize,
    output_index: usize,
    amount_in: u128,
    amp: u128,
    fee_bps: u16,
) -> Result<(u128, u128)> {
    require!(amount_in > 0, StableSwapError::ZeroAmount);

    let new_reserve_in = reserves[input_index]
        .checked_add(amount_in)
        .ok_or(StableSwapError::MathOverflow)?;

    let new_reserve_out = compute_y(reserves, input_index, output_index, new_reserve_in, amp)?;

    let amount_out_before_fee = reserves[output_index]
        .checked_sub(new_reserve_out)
        .ok_or(StableSwapError::MathOverflow)?;

    // Calculate fee
    let fee_amount = amount_out_before_fee
        .checked_mul(fee_bps as u128)
        .ok_or(StableSwapError::MathOverflow)?
        .checked_div(MAX_FEE_BPS as u128)
        .ok_or(StableSwapError::MathOverflow)?;

    let amount_out = amount_out_before_fee
        .checked_sub(fee_amount)
        .ok_or(StableSwapError::MathOverflow)?;

    Ok((amount_out, fee_amount))
}
