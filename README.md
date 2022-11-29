# serde-partial

One of the few things that still require boilerplate when using Serde is partial serialization.

Let's say we have an API route which returns product information. We want to return the stock only to admins and not to visitors. There are a few options here; have a second struct with a subset of the fields which also derives `Serialize`, make the stock field an `Option` with `#[serde(skip_serializing_if = "Option::is_none")]` and set to `None` for visitors, or use something like the `serde_json::json!` macro and do manual serialization.

None of these options are particularly attractive. Having to maintain a struct and handle conversion for each subset of fields is a lot of boilerplate. Making a field that's always present optional just for serialization is a hack at best. Manual serialization kind of defeats the purpose of serde derives.

`serde-partial` aims to make partial serialization (almost) as clean an concise as complete serialization while also being `no_std` compatible. Using this crate this problem could be solved in a single line.

```rust
use serde::Serialize;
use serde_partial::SerializePartial;

#[derive(Serialize, SerializePartial)]
pub struct Product {
    name: String,
    price: u32,
    stock: u32,
}

fn get_product(id: i32) -> Product {
    todo!()
}

fn product_api(id: i32, is_manager: bool) -> String {
    let product = get_product(id);
    if is_manager {
        serde_json::to_string(&product).unwrap()
    } else {
        serde_json::to_string(&product.without_fields(|p| [p.stock])).unwrap()
    }
}
```

Check out the [`with_fields`](SerializePartial::with_fields) [documentation](https://docs.rs/serde-partial) for more details and examples.
