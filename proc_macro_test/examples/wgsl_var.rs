use std::fmt::{Debug, Display};
use std::ops::{Add, Mul};
use std::rc::Rc;

// Internal expression representation
#[derive(Debug, Clone)]
enum WGSLExpr {
    Literal(f32),
    Variable(String),
    BinaryOp {
        left: Box<WGSLExpr>,
        op: String,
        right: Box<WGSLExpr>,
    },
}

// Internal variable representation using Rc
#[derive(Debug, Clone)]
struct WGSLVar {
    init_expr: WGSLExpr,
    ref_expr: WGSLExpr,
    var_name: Option<String>,
}

// Public expression type with implicit cloning
#[derive(Debug, Clone)]
pub struct Expr(Rc<WGSLVar>);

// Builder for creating WGSL code
#[derive(Default)]
pub struct WGSLBuilder {
    vars: Vec<Rc<WGSLVar>>,
}

impl WGSLBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn var(&mut self, name: &str, value: f32) -> Expr {
        let var = Rc::new(WGSLVar {
            init_expr: WGSLExpr::Literal(value),
            ref_expr: WGSLExpr::Variable(name.to_string()),
            var_name: Some(name.to_string()),
        });
        self.vars.push(var.clone());
        Expr(var)
    }

    pub fn literal(&self, value: f32) -> Expr {
        Expr(Rc::new(WGSLVar {
            init_expr: WGSLExpr::Literal(value),
            ref_expr: WGSLExpr::Literal(value),
            var_name: None,
        }))
    }

    pub fn assign(&mut self, name: &str, expr: Expr) -> Expr {
        let var = Rc::new(WGSLVar {
            init_expr: expr.0.ref_expr.clone(),
            ref_expr: WGSLExpr::Variable(name.to_string()),
            var_name: Some(name.to_string()),
        });
        self.vars.push(var.clone());
        Expr(var)
    }

    pub fn build(&self) -> String {
        let declarations: Vec<String> = self
            .vars
            .iter()
            .filter_map(|var| {
                var.var_name
                    .as_ref()
                    .map(|name| format!("let {} = {};", name, wgsl_expr_to_string(&var.init_expr)))
            })
            .collect();

        declarations.join("\n")
    }
}

fn wgsl_expr_to_string(expr: &WGSLExpr) -> String {
    match expr {
        WGSLExpr::Literal(val) => val.to_string(),
        WGSLExpr::Variable(name) => name.clone(),
        WGSLExpr::BinaryOp { left, op, right } => {
            format!(
                "({} {} {})",
                wgsl_expr_to_string(left),
                op,
                wgsl_expr_to_string(right)
            )
        }
    }
}

impl Add<&Expr> for &Expr {
    type Output = Expr;

    fn add(self, rhs: &Expr) -> Self::Output {
        let binary_op = WGSLExpr::BinaryOp {
            left: Box::new(self.0.ref_expr.clone()),
            op: "+".to_string(),
            right: Box::new(rhs.0.ref_expr.clone()),
        };
        Expr(Rc::new(WGSLVar {
            init_expr: binary_op.clone(),
            ref_expr: binary_op,
            var_name: None,
        }))
    }
}

impl Mul<&Expr> for &Expr {
    type Output = Expr;

    fn mul(self, rhs: &Expr) -> Self::Output {
        let binary_op = WGSLExpr::BinaryOp {
            left: Box::new(self.0.ref_expr.clone()),
            op: "*".to_string(),
            right: Box::new(rhs.0.ref_expr.clone()),
        };
        Expr(Rc::new(WGSLVar {
            init_expr: binary_op.clone(),
            ref_expr: binary_op,
            var_name: None,
        }))
    }
}

// Example usage showing cleaner syntax without clones
fn main() {
    let mut wgsl = WGSLBuilder::new();

    let x = wgsl.var("x", 3.0);
    let y = wgsl.var("y", 4.0);

    let r1 = wgsl.assign("r", &x + &(&y * &x));
    let r2 = wgsl.assign("r", &r1 + &x);

    // todo, we need to allow for type and struct definitions and figure out initialization

    // this would be the rust type definition
    type Radius = f32;
    // this is how it would look in wgsl
    // alias Radius = f32;

    // this would be the rust type definition
    struct Position(pub [f32; 2]);
    // this is how it would look in wgsl
    // type Position = array<f32,2>;

    // this would be the rust initialization code
    let v1 = Position([1.0, 2.0]);
    // this is how it would look in wgsl
    // let v1 = Position([1.0, 2.0]);

    // this would be the rust type definition
    struct CollisionResult {
        entity1: u32,
        entity2: u32,
    }
    // this is how it would look in wgsl
    // type CollisionResult = {
    //     entity1: u32,
    //     entity2: u32,
    // };

    // this would be the rust initialization code
    let defined_outside = CollisionResult {
        entity1: x,
        entity2: y,
    };
    // this is how it would look in wgsl
    // let defined_outside = CollisionResult (x, y);

    println!("{}", wgsl.build());
}
