A CLI tool for manipulating factorio blueprint strings.

Usage:

```
Usage: factorio-bp-helper <COMMAND>

Commands:
  count-entities   Counts the number of items needed to construct the blueprint
  unwrap           Unwraps a blueprint string to reveal the json representation
  wrap             Wraps json from stdin into a blueprint string
  upgrade-quality  Upgrades the quality of recipies/filters/conditions without upgrading entities/modules
  save             Saves blueprint as a .json file, or as a directory of json files if it's a blueprint book
  help             Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## License

Copyright 2025 Daniel Johnson et al.

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE2](LICENSE-APACHE2) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
