# Security policy

RustCut runs every plugin in a WebAssembly sandbox: a plugin has no access to files, the network, environment
variables or other programs, and each call has a time limit and each plugin a memory limit. A plugin can only answer
RustCut's events with the actions of the plugin API (set the target folder, set a rating, show a message).

If you find a way for a plugin to get past these limits, or a problem in a plugin of this repository, please do not
open a public issue. Use GitHub's private reporting instead:
[Report a vulnerability](https://github.com/vommie/rustcut-plugins/security/advisories/new) (the **Security** tab of
this repository). Only the newest release of each plugin gets fixes.

Only install plugin files you trust, and compare a downloaded file with its `.sha256` file:

```bash
sha256sum -c auto-folder-1.0.0.rcplugin.sha256
```
