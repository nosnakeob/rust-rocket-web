use syn::Expr::Path;
use syn::visit_mut::VisitMut;
use syn::{ExprCall, ExprPath};

pub struct RbatisConn;

impl VisitMut for RbatisConn {
    // xxx::xxx
    fn visit_expr_call_mut(&mut self, call: &mut ExprCall) {
        if let Path(ExprPath { ref path, .. }) = *call.func
            && let Some(ident) = path.segments.last()
        {
            let ident_str = ident.ident.to_string();
            if ["select", "insert", "update"]
                .iter()
                .any(|&x| ident_str.starts_with(x))
            {
                call.args.insert(0, parse_quote!(rb));
            }
        }

        // eprintln!("visit_expr_call_mut: {:?}", call.to_token_stream().to_string());
    }

    // xxx.xxx
    // fn visit_expr_method_call_mut(&mut self, call: &mut ExprMethodCall) {
    //     eprintln!("visit_expr_method_call_mut: {:?}", call.to_token_stream().to_string());
    // }
}

pub struct Transaction;

impl VisitMut for Transaction {
    fn visit_expr_call_mut(&mut self, call: &mut ExprCall) {
        if let Path(ExprPath { ref path, .. }) = *call.func
            && let Some(ident) = path.segments.last()
        {
            let ident_str = ident.ident.to_string();
            if ["select", "insert", "update"]
                .iter()
                .any(|&x| ident_str.starts_with(x))
            {
                call.args.insert(0, parse_quote!(tx));
            }
        }
    }
}
