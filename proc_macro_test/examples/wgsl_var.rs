use std::fmt::{Debug, Display};
use std::ops::{Add, Mul};
use std::rc::Rc;

// Extended expression representation to include types and structs
#[derive(Debug, Clone)]
enum WGSLExpr {
    Literal(f32),
    Variable(String),
    BinaryOp {
        left: Box<WGSLExpr>,
        op: String,
        right: Box<WGSLExpr>,
    },
    ArrayLiteral(Vec<WGSLExpr>),
    StructLiteral {
        name: String,
        fields: Vec<WGSLExpr>,
    },
}

// Type system representation
#[derive(Debug, Clone)]
enum WGSLType {
    Basic(String), // f32, u32, etc.
    Array {
        element_type: Box<WGSLType>,
        size: usize,
    },
    Struct {
        name: String,
        fields: Vec<(String, WGSLType)>,
    },
    Alias {
        name: String,
        target_type: Box<WGSLType>,
    },
}

// Internal variable representation using Rc
#[derive(Debug, Clone)]
struct WGSLVar {
    init_expr: WGSLExpr,
    ref_expr: WGSLExpr,
    var_name: Option<String>,
    var_type: Option<WGSLType>,
}

// Public expression type with implicit cloning
#[derive(Debug, Clone)]
pub struct Expr(Rc<WGSLVar>);

// Builder for creating WGSL code
#[derive(Default)]
pub struct WGSLBuilder {
    vars: Vec<Rc<WGSLVar>>,
    types: Vec<WGSLType>,
}

impl WGSLBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    // Define a type alias
    pub fn type_alias(&mut self, name: &str, target_type: &str) {
        self.types.push(WGSLType::Alias {
            name: name.to_string(),
            target_type: Box::new(WGSLType::Basic(target_type.to_string())),
        });
    }

    // Define an array type
    pub fn array_type(&mut self, name: &str, element_type: &str, size: usize) {
        self.types.push(WGSLType::Array {
            element_type: Box::new(WGSLType::Basic(element_type.to_string())),
            size,
        });
    }

    // Define a struct type
    pub fn struct_type(&mut self, name: &str, fields: Vec<(&str, &str)>) {
        self.types.push(WGSLType::Struct {
            name: name.to_string(),
            fields: fields
                .into_iter()
                .map(|(name, type_)| (name.to_string(), WGSLType::Basic(type_.to_string())))
                .collect(),
        });
    }

    pub fn var(&mut self, name: &str, value: f32) -> Expr {
        let var = Rc::new(WGSLVar {
            init_expr: WGSLExpr::Literal(value),
            ref_expr: WGSLExpr::Variable(name.to_string()),
            var_name: Some(name.to_string()),
            var_type: Some(WGSLType::Basic("f32".to_string())),
        });
        self.vars.push(var.clone());
        Expr(var)
    }

    pub fn array(&mut self, name: &str, values: Vec<f32>) -> Expr {
        let elements: Vec<WGSLExpr> = values.into_iter().map(WGSLExpr::Literal).collect();
        let var = Rc::new(WGSLVar {
            init_expr: WGSLExpr::ArrayLiteral(elements.clone()),
            ref_expr: WGSLExpr::Variable(name.to_string()),
            var_name: Some(name.to_string()),
            var_type: Some(WGSLType::Array {
                element_type: Box::new(WGSLType::Basic("f32".to_string())),
                size: elements.len(),
            }),
        });
        self.vars.push(var.clone());
        Expr(var)
    }

    pub fn struct_instance(&mut self, name: &str, struct_name: &str, fields: Vec<Expr>) -> Expr {
        let var = Rc::new(WGSLVar {
            init_expr: WGSLExpr::StructLiteral {
                name: struct_name.to_string(),
                fields: fields.iter().map(|e| e.0.ref_expr.clone()).collect(),
            },
            ref_expr: WGSLExpr::Variable(name.to_string()),
            var_name: Some(name.to_string()),
            var_type: Some(WGSLType::Basic(struct_name.to_string())),
        });
        self.vars.push(var.clone());
        Expr(var)
    }

    pub fn build(&self) -> String {
        let mut output = String::new();

        // Type definitions
        for type_def in &self.types {
            output.push_str(&wgsl_type_to_string(type_def));
            output.push_str("\n");
        }

        if !self.types.is_empty() {
            output.push_str("\n");
        }

        // Variable declarations
        let declarations: Vec<String> = self
            .vars
            .iter()
            .filter_map(|var| {
                var.var_name
                    .as_ref()
                    .map(|name| format!("let {} = {};", name, wgsl_expr_to_string(&var.init_expr)))
            })
            .collect();

        output.push_str(&declarations.join("\n"));
        output
    }
}

fn wgsl_type_to_string(type_def: &WGSLType) -> String {
    match type_def {
        WGSLType::Alias { name, target_type } => {
            format!("alias {} = {};", name, wgsl_type_to_string(target_type))
        }
        WGSLType::Array { element_type, size } => {
            format!("array<{}, {}>", wgsl_type_to_string(element_type), size)
        }
        WGSLType::Struct { name, fields } => {
            let fields_str = fields
                .iter()
                .map(|(name, type_)| format!("    {}: {}", name, wgsl_type_to_string(type_)))
                .collect::<Vec<_>>()
                .join(",\n");
            format!("struct {} {{\n{}\n}}", name, fields_str)
        }
        WGSLType::Basic(name) => name.clone(),
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
        WGSLExpr::ArrayLiteral(elements) => {
            let elements_str = elements
                .iter()
                .map(wgsl_expr_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}]", elements_str)
        }
        WGSLExpr::StructLiteral { name, fields } => {
            let fields_str = fields
                .iter()
                .map(wgsl_expr_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", name, fields_str)
        }
    }
}

// Example usage
fn main() {
    let mut wgsl = WGSLBuilder::new();

    // Type alias example
    wgsl.type_alias("Radius", "f32");

    // Array type example
    wgsl.array_type("Position", "f32", 2);
    let pos = wgsl.array("v1", vec![1.0, 2.0]);

    // Struct type example
    wgsl.struct_type("CollisionResult", vec![
        ("entity1", "u32"),
        ("entity2", "u32"),
    ]);

    // Create some variables
    let x = wgsl.var("x", 3.0);
    let y = wgsl.var("y", 4.0);

    // Create a struct instance
    let collision = wgsl.struct_instance("defined_outside", "CollisionResult", vec![x, y]);

    println!("{}", wgsl.build());
}
