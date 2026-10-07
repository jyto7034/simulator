//! Read-only Rust syntax boundary extraction. This does not expand macros,
//! execute cfgs, restore source, or infer an item from a following brace.
use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};
use quote::ToTokens;
use serde_json::{json, Value};
use std::{collections::{BTreeMap, BTreeSet}, io::{self, Read}, str::FromStr};
use syn::{parse::Parser, punctuated::Punctuated, spanned::Spanned, visit::{self, Visit}, Attribute, Meta, Token};

#[derive(Clone)]
enum Cfg { Test, Atom(String), Any(Vec<Cfg>), All(Vec<Cfg>), Not(Box<Cfg>) }
impl Cfg {
    fn parse(m: &Meta) -> Result<Self, String> {
        match m {
            Meta::Path(p) if p.is_ident("test") => Ok(Self::Test),
            Meta::Path(_) | Meta::NameValue(_) => Ok(Self::Atom(m.to_token_stream().to_string())),
            Meta::List(l) => {
                let args = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(l.tokens.clone()).map_err(|e| e.to_string())?;
                let args = args.iter().map(Self::parse).collect::<Result<Vec<_>, _>>()?;
                if l.path.is_ident("any") { Ok(Self::Any(args)) }
                else if l.path.is_ident("all") { Ok(Self::All(args)) }
                else if l.path.is_ident("not") && args.len() == 1 { Ok(Self::Not(Box::new(args.into_iter().next().unwrap()))) }
                else { Err(format!("unsupported cfg operator {}", l.path.to_token_stream())) }
            }
        }
    }
    fn atoms(&self, out: &mut BTreeSet<String>) {
        match self { Self::Atom(a) => { out.insert(a.clone()); }, Self::Any(v)|Self::All(v) => for x in v { x.atoms(out); }, Self::Not(v) => v.atoms(out), Self::Test => {} }
    }
    fn eval(&self, test: bool, values: &BTreeMap<String, bool>) -> bool {
        match self { Self::Test => test, Self::Atom(a) => values[a], Self::Any(v) => v.iter().any(|x| x.eval(test, values)), Self::All(v) => v.iter().all(|x| x.eval(test, values)), Self::Not(v) => !v.eval(test, values) }
    }
    fn influence(&self) -> Result<&'static str, String> {
        let mut atoms = BTreeSet::new(); self.atoms(&mut atoms);
        if atoms.len() > 16 { return Err("cfg predicate exceeds 16 independent non-test atoms; manual review required".into()); }
        let mut positive = false; let mut negative = false;
        for mask in 0..(1usize << atoms.len()) {
            let values = atoms.iter().enumerate().map(|(i,a)| (a.clone(), mask & (1 << i) != 0)).collect();
            let off = self.eval(false, &values); let on = self.eval(true, &values);
            positive |= on && !off; negative |= off && !on;
        }
        Ok(match (positive,negative) { (true,false) => "test_positive", (false,true) => "test_negative_production", (true,true) => "test_mixed", _ => "test_independent" })
    }
}
fn bare_test(ts: TokenStream) -> bool {
    ts.into_iter().any(|t| match t { TokenTree::Ident(i) => i == "test", TokenTree::Group(g) => bare_test(g.stream()), _ => false })
}
fn classify(meta: &Meta) -> Result<Option<&'static str>, String> {
    if meta.path().is_ident("test") { return Ok(Some("test_attribute")); }
    if !meta.path().is_ident("cfg") && !meta.path().is_ident("cfg_attr") { return Ok(None); }
    let Meta::List(l) = meta else { return Err("non-list cfg attribute".into()); };
    if !bare_test(l.tokens.clone()) { return Ok(None); }
    if l.path.is_ident("cfg_attr") { return Err("test-sensitive cfg_attr requires manual expansion review".into()); }
    let inner: Meta = syn::parse2(l.tokens.clone()).map_err(|e| e.to_string())?;
    Cfg::parse(&inner)?.influence().map(Some)
}
fn protected(class: &str) -> bool { matches!(class,"test_positive"|"test_mixed"|"test_attribute") }

struct Scanner<'a> {
    source: &'a str,
    lines: Vec<usize>,
    spans: Vec<Value>,
    conditions: Vec<Value>,
    reviews: Vec<Value>,
    recognized: BTreeSet<usize>,
    context: Vec<String>,
    inline_modules: Vec<String>,
    depth: usize,
    modules: Vec<Value>,
    includes: Vec<Value>,
    macros: Vec<Value>,
}
impl<'a> Scanner<'a> {
    fn offset(&self, p: proc_macro2::LineColumn) -> usize {
        let begin = *self.lines.get(p.line - 1).expect("parser line outside input");
        // proc-macro2 1.0.95 columns count UTF-8 characters, not bytes.
        let rest = &self.source[begin..];
        begin + rest.char_indices().nth(p.column).map_or(rest.len(), |(i,_)| i)
    }
    fn range(&self, span: Span) -> (usize,usize) { (self.offset(span.start()), self.offset(span.end())) }
    fn review(&mut self, start: usize, cause: String) { self.reviews.push(json!({"start":start,"cause":cause})); }
    fn enter(&mut self, attrs: &[Attribute], span: Span, kind: &str) -> bool {
        let (mut start,end) = self.range(span);
        let mut reasons = vec![];
        for a in attrs {
            let (at,_) = self.range(a.span()); start = start.min(at);
            match classify(&a.meta) {
                Ok(Some(class)) => {
                    self.recognized.insert(at);
                    self.conditions.push(json!({"start":at,"class":class,"owner_kind":kind,"context":self.context}));
                    if protected(class) { reasons.push(class); }
                }
                Err(e) => { self.recognized.insert(at); self.review(at,e); }
                Ok(None) => {}
            }
        }
        if reasons.is_empty() { return false; }
        assert!(start <= end && end <= self.source.len());
        self.spans.push(json!({"start":start,"end":end,"kind":kind,"context":self.context,"reasons":reasons}));
        self.depth += 1; true
    }
    fn exit(&mut self, entered: bool) { if entered { self.depth -= 1; } }
    fn module_path(attrs: &[Attribute]) -> Result<Option<String>, String> {
        let mut found = None;
        for a in attrs.iter().filter(|a| a.path().is_ident("path")) {
            let Meta::NameValue(v) = &a.meta else { return Err("non-string module path".into()); };
            let syn::Expr::Lit(v) = &v.value else { return Err("non-literal module path".into()); };
            let syn::Lit::Str(s) = &v.lit else { return Err("non-string module path".into()); };
            if found.replace(s.value()).is_some() { return Err("duplicate module path attributes".into()); }
        }
        Ok(found)
    }
}

// Each method below is generated from syn's actual AST types with attrs fields.
// The syntactic node itself supplies the exact range, including its terminator
// where Rust makes it part of the node. A Variant span never consumes a later
// function; an Arm includes its parsed expression and optional comma.
macro_rules! attrs_visitors {
    ($($method:ident: $ty:ident),* $(,)?) => {$ (
        fn $method(&mut self, n: &'ast syn::$ty) {
            let entered = self.enter(&n.attrs, n.span(), stringify!($ty));
            visit::$method(self,n);
            self.exit(entered);
        }
    )*};
}
impl<'ast> Visit<'ast> for Scanner<'_> {
    fn visit_file(&mut self, n: &'ast syn::File) {
        let entered = self.enter(&n.attrs,n.span(),"File");
        if entered { let last=self.spans.last_mut().unwrap(); last["start"]=json!(0); last["end"]=json!(self.source.len()); }
        visit::visit_file(self,n); self.exit(entered);
    }
    fn visit_item(&mut self, n: &'ast syn::Item) {
        if let syn::Item::Macro(m) = n {
            let (start,end)=self.range(m.span());
            self.macros.push(json!({"start":start,"end":end,"name":m.ident.as_ref().map(ToString::to_string),"path":m.mac.path.to_token_stream().to_string(),"context":self.context}));
        }
        let key = match n {
            syn::Item::Mod(x)=>format!("mod:{}",x.ident), syn::Item::Fn(x)=>format!("fn:{}",x.sig.ident),
            syn::Item::Enum(x)=>format!("enum:{}",x.ident), syn::Item::Struct(x)=>format!("struct:{}",x.ident),
            syn::Item::Impl(x)=>format!("impl:{}:{}",x.trait_.as_ref().map(|(_,p,_)|p.to_token_stream().to_string()).unwrap_or_default(),x.self_ty.to_token_stream()),
            syn::Item::Trait(x)=>format!("trait:{}",x.ident), syn::Item::Type(x)=>format!("type:{}",x.ident),
            syn::Item::Union(x)=>format!("union:{}",x.ident), syn::Item::Const(x)=>format!("const:{}",x.ident),
            syn::Item::Static(x)=>format!("static:{}",x.ident), syn::Item::Use(_)=>"use".into(),
            syn::Item::Macro(x)=>format!("macro:{}",x.ident.as_ref().map(ToString::to_string).unwrap_or_else(||x.mac.path.to_token_stream().to_string())),
            syn::Item::Verbatim(t)=>{ if bare_test(t.clone()) { self.review(self.range(n.span()).0,"verbatim item mentions test; unsupported syntax".into()); } "verbatim".into() },
            _=>"other_item".into(),
        };
        self.context.push(key); visit::visit_item(self,n); self.context.pop();
    }
    fn visit_impl_item(&mut self,n:&'ast syn::ImplItem) {
        let key=match n { syn::ImplItem::Fn(x)=>format!("method:{}",x.sig.ident),syn::ImplItem::Const(x)=>format!("const:{}",x.ident),syn::ImplItem::Type(x)=>format!("type:{}",x.ident),_=>"impl_member".into() };
        self.context.push(key);visit::visit_impl_item(self,n);self.context.pop();
    }
    fn visit_trait_item(&mut self,n:&'ast syn::TraitItem) {
        let key=match n { syn::TraitItem::Fn(x)=>format!("method:{}",x.sig.ident),syn::TraitItem::Const(x)=>format!("const:{}",x.ident),syn::TraitItem::Type(x)=>format!("type:{}",x.ident),_=>"trait_member".into() };
        self.context.push(key);visit::visit_trait_item(self,n);self.context.pop();
    }
    fn visit_item_mod(&mut self,n:&'ast syn::ItemMod) {
        let entered=self.enter(&n.attrs,n.span(),"ItemMod");
        if n.content.is_none() {
            match Self::module_path(&n.attrs) {
                Ok(path)=>self.modules.push(json!({"name":n.ident.to_string(),"path":path,"inline_modules":self.inline_modules,"test_context":self.depth>0,"start":self.range(n.span()).0})),
                Err(e)=>self.review(self.range(n.span()).0,e),
            }
        }
        self.inline_modules.push(n.ident.to_string()); visit::visit_item_mod(self,n); self.inline_modules.pop(); self.exit(entered);
    }
    fn visit_macro(&mut self,n:&'ast syn::Macro) {
        if let Some(name)=n.path.get_ident().map(ToString::to_string) {
            if matches!(name.as_str(),"include"|"include_str"|"include_bytes") {
                let path=syn::parse2::<syn::LitStr>(n.tokens.clone()).ok().map(|s|s.value());
                self.includes.push(json!({"macro":name,"literal_path":path,"tokens":n.tokens.to_string(),"test_context":self.depth>0,"start":self.range(n.span()).0}));
            }
        }
        visit::visit_macro(self,n);
    }
    attrs_visitors! {
        visit_arm: Arm,
        visit_bare_fn_arg: BareFnArg,
        visit_bare_variadic: BareVariadic,
        visit_const_param: ConstParam,
        visit_expr_array: ExprArray,
        visit_expr_assign: ExprAssign,
        visit_expr_async: ExprAsync,
        visit_expr_await: ExprAwait,
        visit_expr_binary: ExprBinary,
        visit_expr_block: ExprBlock,
        visit_expr_break: ExprBreak,
        visit_expr_call: ExprCall,
        visit_expr_cast: ExprCast,
        visit_expr_closure: ExprClosure,
        visit_expr_const: ExprConst,
        visit_expr_continue: ExprContinue,
        visit_expr_field: ExprField,
        visit_expr_for_loop: ExprForLoop,
        visit_expr_group: ExprGroup,
        visit_expr_if: ExprIf,
        visit_expr_index: ExprIndex,
        visit_expr_infer: ExprInfer,
        visit_expr_let: ExprLet,
        visit_expr_lit: ExprLit,
        visit_expr_loop: ExprLoop,
        visit_expr_macro: ExprMacro,
        visit_expr_match: ExprMatch,
        visit_expr_method_call: ExprMethodCall,
        visit_expr_paren: ExprParen,
        visit_expr_path: ExprPath,
        visit_expr_range: ExprRange,
        visit_expr_raw_addr: ExprRawAddr,
        visit_expr_reference: ExprReference,
        visit_expr_repeat: ExprRepeat,
        visit_expr_return: ExprReturn,
        visit_expr_struct: ExprStruct,
        visit_expr_try: ExprTry,
        visit_expr_try_block: ExprTryBlock,
        visit_expr_tuple: ExprTuple,
        visit_expr_unary: ExprUnary,
        visit_expr_unsafe: ExprUnsafe,
        visit_expr_while: ExprWhile,
        visit_expr_yield: ExprYield,
        visit_field: Field,
        visit_field_pat: FieldPat,
        visit_field_value: FieldValue,
        visit_foreign_item_fn: ForeignItemFn,
        visit_foreign_item_macro: ForeignItemMacro,
        visit_foreign_item_static: ForeignItemStatic,
        visit_foreign_item_type: ForeignItemType,
        visit_impl_item_const: ImplItemConst,
        visit_impl_item_fn: ImplItemFn,
        visit_impl_item_macro: ImplItemMacro,
        visit_impl_item_type: ImplItemType,
        visit_item_const: ItemConst,
        visit_item_enum: ItemEnum,
        visit_item_extern_crate: ItemExternCrate,
        visit_item_fn: ItemFn,
        visit_item_foreign_mod: ItemForeignMod,
        visit_item_impl: ItemImpl,
        visit_item_macro: ItemMacro,
        visit_item_static: ItemStatic,
        visit_item_struct: ItemStruct,
        visit_item_trait: ItemTrait,
        visit_item_trait_alias: ItemTraitAlias,
        visit_item_type: ItemType,
        visit_item_union: ItemUnion,
        visit_item_use: ItemUse,
        visit_lifetime_param: LifetimeParam,
        visit_local: Local,
        visit_pat_ident: PatIdent,
        visit_pat_or: PatOr,
        visit_pat_paren: PatParen,
        visit_pat_reference: PatReference,
        visit_pat_rest: PatRest,
        visit_pat_slice: PatSlice,
        visit_pat_struct: PatStruct,
        visit_pat_tuple: PatTuple,
        visit_pat_tuple_struct: PatTupleStruct,
        visit_pat_type: PatType,
        visit_pat_wild: PatWild,
        visit_receiver: Receiver,
        visit_stmt_macro: StmtMacro,
        visit_trait_item_const: TraitItemConst,
        visit_trait_item_fn: TraitItemFn,
        visit_trait_item_macro: TraitItemMacro,
        visit_trait_item_type: TraitItemType,
        visit_type_param: TypeParam,
        visit_variadic: Variadic,
        visit_variant: Variant,
    }
}

// Independently inventory attribute token trees, including opaque macros. This
// prevents a successful AST parse from silently omitting an unvisited cfg.
fn inventory(ts:TokenStream, s:&mut Scanner<'_>) {
    let v:Vec<_>=ts.into_iter().collect();
    for (i,t) in v.iter().enumerate() {
        if let TokenTree::Punct(p)=t {
            if p.as_char()=='#' {
                let mut at=i+1;
                if matches!(v.get(at),Some(TokenTree::Punct(p)) if p.as_char()=='!') { at+=1; }
                if let Some(TokenTree::Group(g))=v.get(at) {
                    if g.delimiter()==Delimiter::Bracket {
                        if let Ok(meta)=syn::parse2::<Meta>(g.stream()) {
                            let start=s.range(p.span()).0;
                            let relevant=match classify(&meta) { Ok(Some(c))=>protected(c),Err(_)=>true,_=>false };
                            let covered=s.spans.iter().any(|v|v["start"].as_u64().unwrap()<=start as u64 && (start as u64)<v["end"].as_u64().unwrap());
                            if relevant && !s.recognized.contains(&start) && !covered { s.review(start,"test-sensitive attribute inside opaque/unhandled syntax; no body boundary inferred".into()); }
                        }
                    }
                }
            }
        }
        if let TokenTree::Group(g)=t { inventory(g.stream(),s); }
    }
}
fn scan(id:&str,source:&str)->Value {
    let ast=match syn::parse_file(source) { Ok(a)=>a,Err(e)=>return json!({"id":id,"parse_error":e.to_string(),"spans":[],"reviews":[{"cause":"Rust parser rejected file; no preservation inference"}]}) };
    let mut lines=vec![0]; lines.extend(source.bytes().enumerate().filter_map(|(i,b)|(b==b'\n').then_some(i+1)));
    let mut s=Scanner { source,lines,spans:vec![],conditions:vec![],reviews:vec![],recognized:BTreeSet::new(),context:vec![],inline_modules:vec![],depth:0,modules:vec![],includes:vec![],macros:vec![] };
    s.visit_file(&ast);
    match TokenStream::from_str(source) { Ok(ts)=>inventory(ts,&mut s),Err(e)=>s.review(0,format!("token inventory failed: {e}")) }
    s.spans.sort_by_key(|v|(v["start"].as_u64().unwrap(),v["end"].as_u64().unwrap()));
    json!({"id":id,"parse_error":null,"spans":s.spans,"conditions":s.conditions,"reviews":s.reviews,"modules":s.modules,"includes":s.includes,"macros":s.macros})
}
fn main() {
    let mut input=String::new(); io::stdin().read_to_string(&mut input).unwrap();
    let input:Value=serde_json::from_str(&input).unwrap();
    let rows:Vec<_>=input["files"].as_array().unwrap().iter().map(|v|scan(v["id"].as_str().unwrap(),v["source"].as_str().unwrap())).collect();
    println!("{}",json!({"schema":"u09_cfg_ast_spans_v1","parser":"syn=2.0.104; proc-macro2=1.0.95 span-locations","files":rows}));
}
