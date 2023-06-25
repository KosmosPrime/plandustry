# mindus
Mindus is a library for working with [Mindustry](https://github.com/Anuken/Mindustry) formats.

## Usage

```rs
use mindus::*;
let reg = build_registry();
let mut ss = SchematicSerializer(&reg);
let s = ss.deserialize_base64("bXNjaAF4nD3SQW6DMBBA0bE94wF104vkDr1H1QVtWUQioTL0/oFJ/Fl9GXiy5ZFBhiJ6n26zvE9tv7T1f5/bZbtNyyJvv/P2065/+3W9i0hdpu952SR/fiWp29qOL4/lDzkfExkiEpWPGqMKpZRRlT/8VQkv4aXwnlUopYw6vRTVvRzeGJVYy1ShlDKqezk8O8+DV/AKXgkvRSllvK2sdU/xFE/xFE/xFE/xNLzxeRlU9wzPOK9xXsMzPMOr3EcNL0VlqlBKGVWpfh+O5+zPmRdnXpx5cebFmRd/eQ9KIReL")?;
let output = Renderer::render(&s);
output.save("output.png");
```

This produces:

![image](https://raw.githubusercontent.com/bend-n/mindus/master/.github/example.png)