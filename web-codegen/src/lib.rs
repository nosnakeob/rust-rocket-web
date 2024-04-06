#![feature(proc_macro_span)]
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro::{Span, TokenStream};
use std::fs;
use std::ops::Add;
use std::str::FromStr;

use syn::{Expr, ItemFn, LitStr, parse_file, Stmt};
use syn::parse_quote;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;

use crate::utils::path2module_path;
use crate::visitor::*;

mod visitor;
mod utils;

#[proc_macro_attribute]
pub fn has_permit(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn); // 我们传入的是一个函数，所以要用到ItemFn
    let func_vis = &func.vis; // pub
    let func_block = &func.block;//

    let func_decl = &func.sig; // 函数申明
    let func_name = &func_decl.ident; // 函数名
    let func_asyncness = &func_decl.asyncness; // 函数名
    let func_generics = &func_decl.generics; // 函数泛型
    let func_inputs = &func_decl.inputs; // 函数输入参数
    let func_output = &func_decl.output; // 函数返回

    let permit = attr.to_string();

    quote!( // 重新构建函数执行
        #func_vis #func_asyncness fn #func_name #func_generics(#func_inputs) #func_output{
            #func_block
            // match crate::token_auth::check_permit(req_in_permit, #s).await {//fixme 判断参数中是否存在httpRequest，以后再说
            //      None =>  #func_block
            //  Some(res) => { return res.resp_json(); }
            // }
            println!(#permit);
        }
    ).into()
}

#[proc_macro_attribute]
pub fn loggedin(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as ItemFn); // 我们传入的是一个函数，所以要用到ItemFn

    let func_decl = &mut func.sig; // 函数申明
    let func_inputs = &mut func_decl.inputs; // 函数输入参数

    // eprintln!("--------------------------");
    // eprintln!("func_inputs: {:?}", func_inputs);
    // 加输入参数引入登录请求守护
    func_inputs.push(parse_quote!(_user_claim: UserClaim));
    // eprintln!("func_inputs: {:?}", func_inputs);

    // 重新构建函数执行
    let new_fn = quote!( #func );

    // eprintln!("new_fn: {}", new_fn);

    new_fn.into()
}

#[proc_macro_attribute]
pub fn rb_conn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as ItemFn);

    func.sig.inputs.push(parse_quote!(rb: &rocket::State<rbatis::RBatis>));

    RbatisConn.visit_item_fn_mut(&mut func);

    func.block.stmts.insert(0, parse_quote! {
        let rb = &**rb;
    });

    // 重新构建函数执行
    let new_fn = quote!( #func );

    // eprintln!("new_fn: {}", new_fn);

    new_fn.into()
}

#[proc_macro]
pub fn rocket_base_path(input: TokenStream) -> TokenStream {
    let base_path = parse_macro_input!(input as LitStr);
    // eprintln!("input: {:?}", base_path);

    let source_path = Span::call_site().source_file().path();
    let content = fs::read_to_string(source_path).unwrap();
    let ast = parse_file(&content).unwrap();

    let mut visitor = RocketRouteFnVisitor::new();
    visitor.visit_file(&ast);
    // eprintln!("route_fns: {:?}", visitor.route_fns);

    let route_fns = visitor.route_fns;

    let new_fn = quote!(
        pub fn routes() -> rocket::fairing::AdHoc {
            rocket::fairing::AdHoc::on_ignite(#base_path, |rocket| async {
                rocket.mount(
                    #base_path,
                    routes![ #(#route_fns),* ]
                )
            })
        }

        const BASE: &str = #base_path;
    );

    // eprintln!("{}", new_fn);

    new_fn.into()
}

#[proc_macro_attribute]
pub fn auto_mount(attr: TokenStream, item: TokenStream) -> TokenStream {
    let dir = parse_macro_input!(attr as LitStr).value();
    // eprintln!("dir: {}", dir);

    let mut func = parse_macro_input!(item as ItemFn);


    if let (Some(Stmt::Expr(Expr::MethodCall(method), _)), Ok(mut entry)) =
        (func.block.stmts.last_mut(), fs::read_dir(&dir)) {
        while let Some(Ok(f)) = entry.next() {
            // let route_path = path2module_path(&mut f.path()) + "::routes()";
            let route_path = proc_macro2::TokenStream::from_str(path2module_path(&mut f.path()).add("::routes()").as_str()).unwrap();

            *method = parse_quote! { #method
                    .attach(#route_path)
                }
        }
    }


    let new_fn = quote!( #func );

    // eprintln!("{}", new_fn);

    new_fn.into()
}
