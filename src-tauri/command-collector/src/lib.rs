use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, parse_quote, Attribute, Item, ItemMod, Path, Token, Visibility};

#[proc_macro_attribute]
pub fn collect_all(attr: TokenStream, item: TokenStream) -> TokenStream {
    let vis = parse_macro_input!(attr as Visibility);
    let mut module = parse_macro_input!(item as ItemMod);

    let commands = Commands::collect(&mut module);

    let Commands {
        commands,
        specta_commands,
    } = commands;

    quote! {
        #vis fn invoke_handler() -> fn(::tauri::ipc::Invoke<::tauri::Wry>) -> bool {
            ::tauri::generate_handler![#commands]
        }

        #vis fn commands() -> ::tauri_specta::Commands<::tauri::Wry> {
            ::tauri_specta::collect_commands![#specta_commands]
        }

        #module
    }
    .into()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CommandKind {
    Manual,
    Auto,
}
impl CommandKind {
    fn parse(attr: &Attribute) -> Option<Self> {
        if !attr.path().is_ident("command") {
            return None;
        }

        let meta = attr.meta.require_list().ok()?;
        let kind = meta.tokens.to_string();

        match kind.as_str() {
            "manual" => Some(Self::Manual),
            "auto" => Some(Self::Auto),
            _ => None,
        }
    }

    fn is_specta(self) -> bool {
        matches!(self, Self::Auto)
    }

    fn generated_attrs(self) -> impl Iterator<Item = Attribute> {
        let tauri = parse_quote!(#[::tauri::command]);

        let specta = self.is_specta().then(|| parse_quote!(#[::specta::specta]));

        std::iter::once(tauri).chain(specta)
    }
}

#[derive(Default)]
struct Commands {
    commands: Punctuated<Path, Token![,]>,
    specta_commands: Punctuated<Path, Token![,]>,
}
impl Commands {
    fn collect(module: &mut ItemMod) -> Self {
        let mut commands = Self::default();
        let module_ident = module.ident.clone();

        let Some((_, items)) = module.content.as_mut() else {
            return commands;
        };

        for item in items {
            let Item::Fn(function) = item else {
                continue;
            };

            let path: Path = {
                let function_ident = &function.sig.ident;
                parse_quote!(#module_ident::#function_ident)
            };

            function.attrs =
                Self::transform_attrs(std::mem::take(&mut function.attrs), &path, &mut commands);
        }

        commands
    }

    fn transform_attrs(attrs: Vec<Attribute>, path: &Path, commands: &mut Self) -> Vec<Attribute> {
        let mut result = Vec::with_capacity(attrs.len());

        for attr in attrs {
            let Some(kind) = CommandKind::parse(&attr) else {
                result.push(attr);
                continue;
            };

            commands.push(path, kind);
            result.extend(kind.generated_attrs());
        }

        result
    }

    fn push(&mut self, path: &Path, kind: CommandKind) {
        self.commands.push(path.clone());

        if kind.is_specta() {
            self.specta_commands.push(path.clone());
        }
    }
}
