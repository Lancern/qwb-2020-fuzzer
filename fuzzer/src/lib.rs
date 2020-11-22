extern crate rand;
extern crate rand_pcg;
extern crate serde;

use std::os::raw::c_void;

use rand::distributions::Uniform;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct CommandSpec {
    pub id: i32,
    pub data: Vec<CommandDataSpec>,
}

#[derive(Clone, Debug)]
pub enum CommandDataSpec {
    SInt { min: i64, max: i64 },
    UInt { min: u64, max: u64 },
    Binary { min_len: usize, max_len: usize },
}

impl CommandSpec {
    fn create<R>(&self, rng: &mut R) -> Command
    where
        R: ?Sized + Rng,
    {
        let data = self.data.iter().map(|spec| spec.create(rng)).collect();
        Command { id: self.id, data }
    }
}

impl CommandDataSpec {
    fn create<R>(&self, rng: &mut R) -> CommandData
    where
        R: ?Sized + Rng,
    {
        match self {
            CommandDataSpec::SInt { min, max } => {
                let dist = Uniform::new_inclusive(*min, *max);
                CommandData::SInt(rng.sample(dist))
            }
            CommandDataSpec::UInt { min, max } => {
                let dist = Uniform::new_inclusive(*min, *max);
                CommandData::UInt(rng.sample(dist))
            }
            CommandDataSpec::Binary { min_len, max_len } => {
                let dist = Uniform::new_inclusive(*min_len, *max_len);
                let len = rng.sample(dist);
                let mut buf = vec![0u8; len];
                rng.fill_bytes(&mut buf);
                CommandData::Binary(buf)
            }
        }
    }
}

const MUTATE_ADD_CMD_PROB: f64 = 0.3;
const MUTATE_REMOVE_CMD_PROB: f64 = 0.3;
const MUTATE_INT_REGEN_PROB: f64 = 0.2;
const MUTATE_INT_DELTA: u64 = 20;
const MUTATE_BUF_EXTEND_PROB: f64 = 0.3;
const MUTATE_BUF_SPLICE_PROB: f64 = 0.3;

#[derive(Clone, Debug)]
pub struct Input {
    pub commands: Vec<Command>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    fn mutate<R>(&mut self, spec: &[CommandSpec], rng: &mut R)
    where
        R: ?Sized + Rng,
    {
        let pat = rng.gen::<f64>();
        if pat <= MUTATE_ADD_CMD_PROB {
            // Add a new command to the input.
            let spec = random_select(rng, spec);
            let cmd = spec.create(rng);
            let idx = rng.sample(Uniform::new_inclusive(0, self.commands.len()));
            self.commands.insert(idx, cmd);
            return;
        }

        if self.commands.len() > 1 && pat <= MUTATE_ADD_CMD_PROB + MUTATE_REMOVE_CMD_PROB {
            // Remove an existing command from the input.
            let idx = rng.sample(Uniform::new(0, self.commands.len()));
            self.commands.remove(idx);
            return;
        }

        // Mutate an existing command.
        let cmd = random_select_mut(rng, &mut self.commands);
        let spec = spec
            .iter()
            .find(|s| s.id == cmd.id)
            .expect("unknown command ID");
        cmd.mutate(spec, rng);
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Command {
    pub id: i32,
    pub data: Vec<CommandData>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CommandData {
    SInt(i64),
    UInt(u64),
    Binary(Vec<u8>),
}

impl Command {
    fn mutate<R>(&mut self, spec: &CommandSpec, rng: &mut R)
    where
        R: ?Sized + Rng,
    {
        assert_eq!(spec.id, self.id);
        assert_eq!(self.data.len(), spec.data.len());

        let data_idx = rng.sample(Uniform::new(0, self.data.len()));
        match (&mut self.data[data_idx], &spec.data[data_idx]) {
            (CommandData::SInt(value), CommandDataSpec::SInt { min, max }) => {
                mutate_signed_int(value, *min, *max, rng);
            }
            (CommandData::UInt(value), CommandDataSpec::UInt { min, max }) => {
                mutate_unsigned_int(value, *min, *max, rng);
            }
            (CommandData::Binary(value), CommandDataSpec::Binary { min_len, max_len }) => {
                mutate_buf(value, *min_len, *max_len, rng);
            }
            _ => panic!("Command data mismatches with command data spec"),
        };
    }
}

fn mutate_signed_int<R>(value: &mut i64, min: i64, max: i64, rng: &mut R)
where
    R: ?Sized + Rng,
{
    assert!(*value >= min && *value <= max);
    if rng.gen::<f64>() <= MUTATE_INT_REGEN_PROB {
        // Regenerate signed integer.
        *value = rng.sample(Uniform::new_inclusive(min, max));
        return;
    }
    let max_add = std::cmp::min(MUTATE_INT_DELTA as i64, max - *value);
    let max_sub = std::cmp::min(MUTATE_INT_DELTA as i64, *value - min);
    let delta = rng.sample(Uniform::new_inclusive(-max_sub, max_add));
    *value += delta;
}

fn mutate_unsigned_int<R>(value: &mut u64, min: u64, max: u64, rng: &mut R)
where
    R: ?Sized + Rng,
{
    assert!(*value >= min && *value <= max);
    if rng.gen::<f64>() <= MUTATE_INT_REGEN_PROB {
        // Regenerate unsigned integer.
        *value = rng.sample(Uniform::new_inclusive(min, max));
        return;
    }
    let max_add = std::cmp::min(MUTATE_INT_DELTA, max - *value);
    let max_sub = std::cmp::min(MUTATE_INT_DELTA, *value - min);
    let delta = rng.sample(Uniform::new_inclusive(-(max_sub as i64), max_add as i64));
    *value = (*value as i64 + delta) as u64;
}

fn mutate_buf<R>(buf: &mut Vec<u8>, min_len: usize, max_len: usize, rng: &mut R)
where
    R: ?Sized + Rng,
{
    assert!(buf.len() >= min_len && buf.len() <= max_len);

    let pat = rng.gen::<f64>();
    let mut acc_prob = 0f64;

    if buf.len() < max_len {
        acc_prob += MUTATE_BUF_EXTEND_PROB;
        if pat <= acc_prob {
            // Extend the buffer.
            let extend_len = rng.sample(Uniform::new_inclusive(0, max_len - buf.len()));
            let mut extend_buf = vec![0u8; extend_len];
            rng.fill_bytes(&mut extend_buf);
            buf.extend_from_slice(&extend_buf);
            return;
        }
    }

    if buf.len() > min_len {
        acc_prob += MUTATE_BUF_SPLICE_PROB;
        if pat <= acc_prob {
            // Splice the buffer.
            let max_splice_len = buf.len() - min_len;
            let splice_begin = rng.sample(Uniform::new(0, buf.len()));
            let splice_end = rng.sample(Uniform::new(
                splice_begin,
                std::cmp::min(splice_begin + max_splice_len, buf.len()),
            )) + 1;
            *buf = buf
                .splice(splice_begin..splice_end, std::iter::empty())
                .collect();
            return;
        }
    }

    if buf.is_empty() {
        return;
    }

    // Perform arithmetic operation on some byte.
    let target = random_select_mut(rng, buf);
    let delta = rng.sample(Uniform::new_inclusive(
        -(MUTATE_INT_DELTA as i8),
        MUTATE_INT_DELTA as i8,
    ));
    let delta = unsafe { std::mem::transmute::<i8, u8>(delta) };
    *target = target.wrapping_add(delta);
}

pub struct Fuzzer {
    afl: *const c_void,
    spec: Vec<CommandSpec>,
    rng: Pcg32,
}

impl Fuzzer {
    pub fn afl(&self) -> *const c_void {
        self.afl
    }

    pub fn spec(&self) -> &[CommandSpec] {
        &self.spec
    }

    pub fn mutate(&mut self, input: &mut Input) {
        input.mutate(&self.spec, &mut self.rng);
    }
}

pub struct FuzzerBuilder {
    afl: *const c_void,
    spec: Vec<CommandSpec>,
    rng_seed: u32,
}

impl FuzzerBuilder {
    pub fn new(afl: *const c_void, rng_seed: u32) -> Self {
        Self {
            afl,
            spec: Vec::new(),
            rng_seed,
        }
    }

    pub fn add_spec(mut self, spec: CommandSpec) -> Self {
        // Sanity checks.
        for data_spec in &spec.data {
            match data_spec {
                CommandDataSpec::SInt { min, max } => {
                    debug_assert!(*min <= *max);
                }
                CommandDataSpec::UInt { min, max } => {
                    debug_assert!(*min <= *max);
                }
                CommandDataSpec::Binary { min_len, max_len } => {
                    debug_assert!(*min_len <= *max_len);
                }
            };
        }

        self.spec.push(spec);
        self
    }

    pub fn build(self) -> Box<Fuzzer> {
        Box::new(Fuzzer {
            afl: self.afl,
            spec: self.spec,
            rng: Pcg32::seed_from_u64(self.rng_seed as u64),
        })
    }
}

fn random_select<'r, 'v, R, T>(rng: &'r mut R, values: &'v [T]) -> &'v T
where
    R: ?Sized + Rng,
{
    let idx = rng.sample(Uniform::new(0, values.len()));
    &values[idx]
}

fn random_select_mut<'r, 'v, R, T>(rng: &'r mut R, values: &'v mut [T]) -> &'v mut T
where
    R: ?Sized + Rng,
{
    let idx = rng.sample(Uniform::new(0, values.len()));
    &mut values[idx]
}
