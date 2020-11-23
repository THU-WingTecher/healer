/// resource oriented generation algorithm
use crate::fuzzer::ValuePool;
use crate::target::Target;
use hlang::ast::*;
use rand::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::rc::Rc;

/// Gnerate test case based current value pool and test target.
pub fn gen(target: &Target, pool: &ValuePool) -> Prog {
    let mut ctx = GenContext {
        target,
        generated_res: FxHashMap::default(),
        generated_buf: FxHashMap::default(),
        pool,
    };
    gen_inner(&mut ctx)
}

/// Generated resource during one generation.
type ResPool = FxHashMap<Rc<Type>, FxHashSet<Rc<ResValue>>>;

/// Generation context.
/// A context contains test target, generated resource and buffer value, global value pool.
struct GenContext<'a, 'b> {
    target: &'a Target,
    generated_res: ResPool,
    generated_buf: FxHashMap<Rc<Type>, Value>,
    pool: &'b ValuePool,
}

fn gen_inner(ctx: &mut GenContext) -> Prog {
    let calls: Vec<Call> = Vec::new();
    while !should_stop(calls.len()) {
        let next_sc = select_syscall(ctx);
    }
    todo!()
}

fn should_stop(len: usize) -> bool {
    const MIN_LEN: usize = 4;
    const MAX_LEN: usize = 16;
    if len < MIN_LEN {
        // not long enough
        false
    } else if len < MAX_LEN {
        // we can continue, we can alse stop, so we use rand.
        let delta = 0.8 * (len as f32 / MAX_LEN as f32);
        random::<f32>() < delta
    } else {
        true
    }
}

/// Select a syscall to fuzz based on resource usage.
fn select_syscall(ctx: &GenContext) -> Rc<Syscall> {
    if should_try_gen_res(ctx) {
        let res_ty = select_res(&ctx.target.res_tys);
        select_res_producer(ctx.target, res_ty)
    } else {
        select_syscall_rand(ctx)
    }
}

fn should_try_gen_res(ctx: &GenContext) -> bool {
    // Since the length of a test case is [4, 16}, the number
    // of generated resource should be [2, 6}
    const MIN_RES_NUMBER: usize = 2;
    const MAX_RES_NUMBER: usize = 6;
    let res_count = ctx.generated_res.len();
    if res_count == 0 {
        // always make sure we start from generating a resource.
        true
    } else if res_count >= MAX_RES_NUMBER {
        random::<f32>() < 0.2 * (MAX_RES_NUMBER as f32 / (res_count as f32 * 2.0))
    } else {
        let alpha = 1.0 - (res_count as f32) / (MAX_RES_NUMBER as f32);
        if res_count < MIN_RES_NUMBER {
            random::<f32>() < 0.8 * alpha
        } else {
            random::<f32>() < 0.4 * alpha
        }
    }
}

fn select_syscall_rand(ctx: &GenContext) -> Rc<Syscall> {
    ctx.target
        .syscalls
        .iter()
        .choose(&mut thread_rng())
        .map(Rc::clone)
        .unwrap()
}

fn select_res(res_tys: &[Rc<Type>]) -> &Type {
    res_tys
        .iter()
        .map(|t| &*t)
        .choose(&mut thread_rng())
        .unwrap()
}

fn select_res_producer(t: &Target, res: &Type) -> Rc<Syscall> {
    let res_desc = res.res_desc().unwrap();
    let eq_class = &t.res_eq_class[res];
    let accurate_ctors = &res_desc.ctors;
    let all_ctors = eq_class
        .iter()
        .flat_map(|res| res.res_desc().unwrap().ctors.iter());
    let mut rng = thread_rng();
    if !accurate_ctors.is_empty() && random::<f32>() < 0.85 {
        accurate_ctors
            .iter()
            .choose(&mut rng)
            .map(Rc::clone)
            .unwrap()
    } else {
        // unreachable resources were removed during constructing target.
        all_ctors.choose(&mut rng).map(Rc::clone).unwrap()
    }
}

fn gen_call(ctx: &mut GenContext, sc: Rc<Syscall>) -> Call {
    todo!()
}
