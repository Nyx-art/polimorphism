use proc_macro::TokenStream;

fn remove_first_last(s: &String) -> String {
    let mut chars = s.chars();
    chars.next();
    chars.next_back();
    String::from(chars.as_str())
}

fn from_args(args: &String, ty: bool) -> String {
    let mut out=String::from("(");
    let tt: TokenStream=remove_first_last(args).parse().unwrap();
    let mut pos=0; //0: ident; 1: ":", 2: ty, 3: ","
    let filter=if ty {2} else {0};
    for tok in tt {
        let s=tok.to_string();
        if s=="&" { pos=2; }
        else if s=="self" {
            pos=0;
            if filter!=pos {
                out+="Self "
            }
        }
        if s==":" {
            pos=2;
        }
        else if s=="," {
            pos=0;
            out+=",";
        }
        else if pos==filter {
            out+=&s;
            if s!="'" {out+=" ";}
        }
    }
    out+=")";
    out
}

fn get_fn_sign(args: &String) -> String {
    let mut s=args.split(',');
    let s0=s.next();
    let mut out=String::new();
    match s0 {
        Some(str) => {
            if str.contains("&") { 
                out+="& ";
                if str.contains("mut") { out+="mut "; }
            }
            return out
        }
        None => { return out }
    }
}

fn parse_function(f: TokenStream) -> Vec<String> {
    let mut out=[();6].into_iter().map(|_| String::new()).collect::<Vec<_>>();
    let tt=f.into_iter().map(|x| x.to_string()).collect::<Vec<_>>();
    let mut time=0;
    let mut depth=0;
    for tok in &tt {
        match time {
            0 => {
                if tok=="<" { time=1; depth+=1; }
                else if tok.contains("(") && out[0].contains("fn") { out[3]+=tok; time=3; }
                else {
                    if out[1].is_empty() && out[0].contains("fn") { out[1]+=tok; }
                    out[0]+=tok; out[0]+=" ";
                }
            }
            1 => {
                if tok=="<" { depth+=1; }
                if tok==">" { 
                    depth-=1; 
                    if depth==0 {time=2;}
                    else {
                        out[2]+=tok;
                        out[2]+=" ";
                    }
                }
                else {
                    out[2]+=tok;
                    if tok!="'" { out[2]+=" "; }
                }
            }
            2 => {
                out[3]+=tok;
                time=3;
            }
            3 => {
                if tok.contains("{") { out[5]=remove_first_last(tok); break; }
                else {
                    out[4]+=tok;
                    if tok!="'" && tok!="-" { out[4]+=" "; }
                }
            }
            _ => {}
        }
    }
    out
}

fn parse_impl(im: TokenStream) -> Vec<String> {
    let mut out=[();2].into_iter().map(|_| String::new()).collect::<Vec<_>>();
    let tt=im.into_iter().map(|x| x.to_string()).collect::<Vec<_>>();
    let mut time=0;
    let is_impl=&tt[1]=="<";
    let mut depth=0;
    for tok in &tt {
        match time {
            0 => {
                if is_impl && tok=="<" { depth+=1; time=1; }
                else if !is_impl { time=2; }
            }
            1 => {
                if tok=="<" { depth+=1; }
                if tok==">" { 
                    depth-=1; 
                    if depth==0 {time=2;}
                    else {
                        out[0]+=tok;
                        out[0]+=" ";
                    }
                }
                else {
                    out[0]+=tok;
                    if tok!="'" {
                        out[0]+=" ";
                    }
                }
            }
            2 => {
                if tok.contains("{") { break;}
                else {
                    out[1]+=tok;
                    if tok!="'" { out[1]+=" "; }
                }
            }
            _ => {}
        }
    }
    out
}

fn get_function(all: TokenStream) -> (TokenStream, TokenStream) {
    let mut f=Vec::new();
    let mut tt=Vec::new();
    let mut is_f=true;
    for tok in all {
        if is_f {
            let s=tok.to_string();
            if s.contains("{") {
                is_f=false;
            }
            f.push(tok);
        }
        else {
            tt.push(tok);
        }
    }
    (f.into_iter().collect(), tt.into_iter().collect())
}

fn get_impl(all: TokenStream) -> (TokenStream, TokenStream) {
    let mut f=Vec::new();
    let mut tt=TokenStream::new();
    let mut is_i=true;
    for tok in all {
        if is_i {
            let s=tok.to_string();
            if s.contains("{") {
                is_i=false;
                tt=remove_first_last(&tok.to_string()).parse().unwrap();
            }
            else { f.push(tok); }
        }
    }
    (f.into_iter().collect(), tt)
}

fn get_type_var(s: &String) -> (String,String) {
    (remove_first_last(&from_args(s, true)),remove_first_last(&from_args(s, false)))
}

fn get_type_var_refer(s: &String) -> (String,String,String) {
    (from_args(s, true),from_args(s, false),get_fn_sign(s))
}

fn merge_impls(i1: &String, i2: &String) -> String {
    if i1.is_empty() && i2.is_empty() {
        return String::new()
    }
    if i1.is_empty() {
        return i2.to_string()
    }
    if i2.is_empty() {
        return i1.to_string()
    }
    let mut im1=i1.split(',').collect::<Vec<_>>();
    let im2=i2.split(',').collect::<Vec<_>>();
    for i in im2 {
        if i.contains("'") {im1.insert(0, i);}
        else {im1.push(i);}
    }
    im1.join(",")
}

fn get_crate_of_call(call: TokenStream) -> (String,Vec<String>) {
    let mut out=String::new();
    let tt=call.into_iter().map(|x| x.to_string()).collect::<Vec<_>>();
    let d=tt.iter().filter(|x| *x==":").collect::<Vec<_>>().len();
    if d%2==0 { return (out,tt) }
    let mut iter=tt.into_iter();
    let mut v_str=vec![];
    loop {
        let t_tok=iter.next();
        if t_tok.is_none() { break; }
        let tok=t_tok.unwrap();
        let mut it=None;
        if tok==":" {
            it=iter.next();
            if it.is_none() { break; }
            if !it.clone().is_some_and(|x| x==":") { v_str.push(it.unwrap()); break; }
            else {
                out+=&tok;
                if it.is_some() { out+=&it.unwrap(); }
            }
        }
        else {
            out+=&tok;
            if it.is_some() { out+=&it.unwrap(); }
        }
    }
    out+="::";
    v_str.extend(iter);
    (out,v_str)
}

static mut DEFINED_POLY_FNS: Vec<String>=vec![];

/// ## Polimorphism
/// See docs at `polymorphism!`
#[proc_macro]
pub fn polimorphism(_item: TokenStream) -> TokenStream {
    let s=_item.clone().into_iter().map(|x| x.to_string()).collect::<Vec<_>>();
    if s.is_empty() {
        let out=format!("
        pub struct Local<T> {{\n
            pub inner: T\n
        }}\n
        impl<T> Local<T> {{\n
            pub fn __poli_new(val: T) -> Self {{\n
                Self {{ inner: (val) }}\n
            }}
        }}");
        return out.parse().unwrap()
    }
    else if s.contains(&String::from("impl")) {
        let mut out=String::new();
        let (im,mut rest)=get_impl(_item);
        let imp=parse_impl(im);
        let mut f;
        let mut fn_names=vec![];
        while !rest.is_empty() {
            (f,rest)=get_function(rest);
            let mut func=parse_function(f);
            if !func[3].contains("self") {
                let (mut ty,mut var)=get_type_var(&func[3]);
                for i in 0..func.len() {
                    func[i]=func[i].replace("Self ", &imp[1]);
                }
                ty=ty.replace("Self ", &imp[1]);
                var=var.replace("Self ", &imp[1]);
                let impl_loc=merge_impls(&imp[0], &func[2]);
                out+=&format!("
                impl<{}> Local<(std::marker::PhantomData<{}>,{})> {{\n
                    {}(self) {} {{\n
                        let Local{{ inner: (_,{})}}=self;\n
                        {}\n
                    }}\n
                }}",impl_loc,imp[1],ty,func[0],func[4],var,func[5]);
            }
            else {
                let (mut ty,mut var,refer)=get_type_var_refer(&func[3]);
                for i in 0..func.len() {
                    func[i]=func[i].replace("Self ", &imp[1]);
                    func[i]=func[i].replace("self", "_self");
                }
                ty=remove_first_last(&ty.replace("Self ", &imp[1]).replace("self", "_self"));
                var=remove_first_last(&var.replace("Self ", &imp[1]).replace("self", "_self"));
                let tr=format!("Polimorphism{}",func[1]);
                if !fn_names.contains(&func[1]) {
                    if !unsafe {DEFINED_POLY_FNS.contains(&func[1])} {
                        out+=&format!("pub trait {} {{\n
                            fn __poliself_{}({}self) -> {}Self\n {{
                                self
                            }}\n
                        }}\n",tr,func[1],refer,refer);
                        unsafe {DEFINED_POLY_FNS.push(func[1].to_string());}
                    }
                    out+=&format!("impl<{}> {} for {} {{}}\n",imp[0],tr,imp[1]);
                    fn_names.push(func[1].to_string());
                }
                let impl_loc=merge_impls(&imp[0], &func[2]);
                out+=&format!("impl<{}> Local<({},)> {{\n
                    {}(self) {} {{\n
                        let Local{{ inner: ({},)}}=self;\n
                        {}\n
                    }}\n
                }}",impl_loc,ty,func[0],func[4],var,func[5]);
            }
        }
        return out.parse().unwrap()
    }
    else if s.contains(&String::from("fn")) {
        let mut out=String::new();
        let mut rest=_item;
        let mut f;
        while !rest.is_empty() {
            (f,rest)=get_function(rest);
            let func=parse_function(f);
            let (ty,var)=get_type_var(&func[3]);
            out+=&format!("impl<{}> Local<({})> {{\n
                {}(self) {} {{\n
                    let Local{{ inner: ({})}}=self;\n
                    {}\n
                }}\n
            }}",func[2],ty,func[0],func[4],var,func[5]);
        }
        return out.parse().unwrap()
    }
    else {
        let (cr,call)=get_crate_of_call(_item);
        if call.contains(&String::from(".")) {
            let mut time=0;
            let mut strc=[();3].map(|_|String::new()).into_iter().collect::<Vec<_>>();
            for tok in call {
                if time==0 {
                    if tok=="." { time=1; }
                    else {
                        strc[0]+=&tok;
                        if &tok!="-" && &tok!="'" {strc[0]+=" ";}
                    }
                }
                else if time==1 {
                    strc[2]=tok;
                    time=2;
                }
                else if time==2 {
                    strc[1]=remove_first_last(&tok);
                }
            }
            let out=format!("{cr}Local::__poli_new(({}.{}(),{})).{}()",strc[0],format!("__poliself_{}",strc[2]),strc[1],strc[2]);
            return out.parse().unwrap()
        }
        else if call.iter().filter(|x| *x==":").collect::<Vec<_>>().len()>1 { //call contains :: after crate_name has been stripped
            let mut time=0;
            let mut strc=[();4].map(|_|String::new()).into_iter().collect::<Vec<_>>();
            for tok in &call {
                if tok==":" {
                    time=1;
                }
                else if time==0 {
                    strc[0]+=tok;
                }
                else if time==1 {
                    strc[2]+=tok;
                    time=2;
                }
                else {
                    strc[1]=remove_first_last(tok);
                }
            }
            let out=format!("{cr}Local::__poli_new((std::marker::PhantomData::<{}>,{})).{}()",strc[0],strc[1],strc[2]);
            return out.parse().unwrap()
        }
        else {
            let res=format!("{cr}Local::__poli_new({}).{}()",call[1],call[0]);
            return res.parse().unwrap()
        }
    } 
}

/// ## Polymorphism
/// A procedural macro to imitate polymorphism which can be seen and found
/// in many modern programming languages. Can be used similarly to an `fn`
/// or `impl` declaration, but `polymorphism` allows for duplicate `fn` names
/// with different signitures (types as parameters). This implementation of
/// `polymorphism` bypasses the orphan rule with a `Local` type.
/// 
/// ## Examples
/// ### - Init:
/// To use `polymorphism!` it needs to be initialised. This can be done with a simple
/// ```
/// polymorphism!();
/// ```
/// closure. Don't initialise twice!
/// ### - Fn Declaration:
/// ```
/// polymorphism!(
///     pub fn func(n: i32, m: i32) -> i32 {
///         n+m
///     }
///     pub fn func(n: f64, m: f64) -> f64 {
///         n-m
///     }
/// );
/// ```
/// ### - Fn Usage:
/// ```
/// assert_eq!(polymorphism!(func(1,2)), 3);
/// assert_eq!(polymorphism!(func(1.0,2.0)), -1.0);
/// ```
/// ### - Impl Declaration:
/// ```
/// polymorphism!(
///     impl<T> Vec<T> {
///         pub fn add_elem(&mut self, elem: T) {
///             self.push(elem);
///         }
///         pub fn add_elem(&mut self, mut elems: Vec<T>) {
///             self.append(&mut elems);
///         }
///     }
/// );
/// ```
/// ### - Impl Usage:
/// ```
/// let mut v=vec![1,2,3];
/// polymorphism!(v.add_elem(4));
/// polymorphism!(v.add_elem(vec![5,6,7]));
/// assert_eq!(v,vec![1,2,3,4,5,6,7]);
/// ```
/// ### - Impl Struct Method Declaration:
/// ```
/// polymorphism!(
///     impl<T> Vec<T> {
///         pub fn new() -> Self {
///             Vec::new()
///         }
///         pub fn new<const N: usize>(arr: [T;N]) -> Self {
///             Vec::from(arr)
///         }
///     }
/// );
/// ```
/// ### - Impl Struct Method Usage:
/// ```
/// let mut v: Vec<i32>=vec![];
/// assert_eq!(polymorphism!(Vec<i32>::new()),v);
/// v.extend([1,2,3]);
/// assert_eq!(polymorphism!(Vec<i32>::new([1,2,3])),v);
/// ```
/// ### - polymorphism From Crate:
/// ```
/// polymorphism!(crate_name: func(1,2));
/// polymorphism!(crate_name: v.add_elem(4));
/// polymorphism!(crate_name: Vec<i32>::new([1,2,3]));
/// ```
/// ## Notes:
/// - You may need to add lifetime specifiers to references (such as `&` or `&mut`)
/// - When using `impl` declaration, functions with the same name should be declared in the same `polymorphism!` closure
/// - Traits don't work on `impl` declarations using `polymorphism!`
/// - All `impl` functions with the same name must have the same reference to `self` (`&self`, `&mut self` or `self`)
#[proc_macro]
pub fn polymorphism(_item: TokenStream) -> TokenStream {
    polimorphism(_item)
}