# Frontend

The frontend is currently generated through [askama templates](https://askama.readthedocs.io/en/stable/) for server-side rendered pages
and uses Web Components for interactive elements.

Normally, content that will be statically served by the frontend module (i.e. stylesheet and web components) is embedded into the binary.
Using the `frontend-dev` feature you can serve it from source to see changes without recompiling RustiCal.
