# Notes

## Kubernetes setup

If you setup RustiCal with Kubernetes and call the deployment `rustical`
Kubernetes will by default expose some environment variables starting with `RUSTICAL_`
that will be rejected by RustiCal.
So for now the solutions are either not calling the deployment `rustical` or setting
`enableServiceLinks: false`, see <https://kubernetes.io/docs/tutorials/services/connect-applications-service/#accessing-the-service>.

For the corresponding issue see <https://github.com/lennart-k/rustical/issues/122>
