# Changelog

All notable changes to ShopWise are documented in this file (NFR-15).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html),
as recommended by the Rust API guidelines
([C-RELNOTES](https://rust-lang.github.io/api-guidelines/documentation.html#release-notes-document-all-significant-changes-c-relnotes)).

## [Unreleased]

Work on branches that are not yet merged into `main`.

### Added
- Map showing the planned route (#75)
- Store locations loaded from the database (#110)
- Store dropdown only lists stores within the maximum range (#85)
- Clear error messages when the price calculator can't find a plan (#82)
- Footer with privacy and data source information (#109)
- CI checks that failed merge requests that don't add a changelog entry, and keeps the Cargo.toml version in sync with the changelog (NFR-15)

### Changed
- Price calculator uses the real scraped product database instead of demo data, with caching (#110)
- Distances follow real roads between stops, using OpenStreetMap routing data instead of straight lines (#81)
- Updated eframe to 0.36; routing data is stored with Git LFS (#75)
- Redesigned logo and browser tab icon (#109)
- Code documentation for the shopping list, stores and file handling (#103)
- Tidied up the results panel code, with no change to how it looks or behaves

### Fixed
- Travel distance and speed calculations (#110)
- Very large mileage values now show a proper error instead of failing (#110)
- Items were removed from the results for a whole brand instead of a single store (#110)
- Web (WASM) build failed to compile (#81)

## [0.7.0] - 2026-09-30

### Added
- Masthead with the ShopWise logo, a link to the GitLab repository and a browser tab icon (#83)
- Masthead button that collapses and expands the filters panel (#84)
- Product search when adding an item: the Add Item button opens a search box with a scrollable list of matching products from the database (#78)

### Changed
- Shopping list rows are read-only, with the remove button at the start of each row; items are added through product search (#78)
- The filters panel and the output panel scroll, and the shopping list and supermarket sections have a fixed maximum height (#76)
- Results show the pack size of each item, e.g. "2x 1L", and line totals use the number of packs (#86)

### Fixed
- Unit was always shown as "each" in the results (#86)
- Item resolver now uses the base unit when scaling quantities (#78)

## [0.6.0] - 2026-09-18

### Added
- Loyalty card checkboxes (Clubcard for New World and Pak'nSave) so member pricing can be used (#28)
- Store location lists read from the `*_locations.json` files, with a dropdown for excluding individual stores (#28)
- Item resolver matches units, including comparing weight with volume, with "each" as a universal unit (#73)
- Item search function that returns the best matching products for a query (#73)

### Changed
- Item resolver works from the selected item instead of a raw search term (#73)
- Refactored the GUI and moved the location input to the side panel; applied Clippy lint fixes and removed dead code (#73)

### Fixed
- Black screen caused by overflowing panels: the shopping list and location dropdown now scroll

## [0.5.0] - 2026-09-11

### Added
- FR-09: output panel connected to the price calculator, showing results (#64)
- FR-13: load and save the shopping list as a CSV file
- FR-01: remove items from and clear the shopping list
- Live address autocomplete and "use current location" with timeout and error handling (#53)
- Distance and travel duration in the results; travel time calculated from mileage speed (#66)
- Hover text explaining the controls on every input panel

### Changed
- Refactored the price calculator to take the transit mileage into account; added `RoutePlan` and `RoutePath` (#66)
- Search button stays disabled until the shopping list and location are valid (#53)
- Location input moved to the right-hand panel (#75)

### Removed
- Filter route debug text (`print_filters()`) (#64)

### Fixed
- Search results not displaying, and the search button failing to enable (#75)
- CSV load bug and file dialog issues (#75)
- Store lookup incorrectly searching at the user's own location (#66)
- GitLab Pages deploy URL (#75)
- Contributor names in `.mailmap` (#79)

## [0.4.0] - 2026-08-28

### Added
- Price calculation using the product database, which runs in the browser (WASM) from memory
- Route planner store filtering by location and supermarket (#20)
- Live website deployed to GitLab Pages on merge to `main`, plus a manual test page job (#60)
- README instructions for running the desktop version (#61, #63)

### Fixed
- Maximum range slider did not update (#37)

### Removed
- Demo artefacts and Nix files that should not have been committed (#36)

Note: the price calculation merge was reverted on 2026-08-23 and merged again on 2026-08-26.

## [0.3.0] - 2026-07-31

### Added
- FR-01: shopping list input in the GUI (#26)
- FR-02: manual location input and "use current location" (#26)
- Mileage (transit cost) selector, with `Cost` arithmetic and formatting (#27)

### Changed
- RACI matrix updated with new requirements after stakeholder validation (#32)
- "ea" is the default unit; the "None" unit was removed (#26)

### Fixed
- Items with the same name sharing one widget id (#26)

## [0.2.0] - 2026-07-21

### Added
- CI pipeline that blocks merging code which fails to compile, fails tests or has warnings (#4)
- FR-10: advanced store filters in the GUI, using egui (#15)
- Web (WASM) support for the GUI (#15)
- FR-03: store filters and the `swizzle_set` proc macro (#12, #18)
- FR-06: price calculation draft (#13)
- `.mailmap` for consistent contributor names (#6)

## [0.1.0] - 2026-05-06

### Added
- RACI matrix (#1)
- Initial Cargo workspace (`shop_wise`, `util`, `database`, `map`) (#3)
