use crate::{Diagnostic, DiagnosticsContext};

// Diagnostic: unresolved-import
//
// This diagnostic is triggered if rust-analyzer is unable to resolve a path in
// a `use` declaration.
pub(crate) fn unresolved_import(
    ctx: &DiagnosticsContext<'_>,
    d: &hir::UnresolvedImport,
) -> Diagnostic {
    Diagnostic::new(
        "unresolved-import",
        "unresolved import",
        ctx.sema.diagnostics_display_range(d.decl.clone().map(|it| it.into())).range,
    )
    // This currently results in false positives in the following cases:
    // - `cfg_if!`-generated code in libstd (we don't load the sysroot correctly)
    // - `core::arch` (we don't handle `#[path = "../<path>"]` correctly)
    // - proc macros and/or proc macro generated code
    .experimental()
}

#[cfg(test)]
mod tests {
    use crate::tests::check_diagnostics;

    #[test]
    fn unresolved_import() {
        check_diagnostics(
            r#"
use does_exist;
use does_not_exist;
  //^^^^^^^^^^^^^^ unresolved import

mod does_exist {}
"#,
        );
    }

    #[test]
    fn unresolved_import_in_use_tree() {
        // Only the relevant part of a nested `use` item should be highlighted.
        check_diagnostics(
            r#"
use does_exist::{Exists, DoesntExist};
                       //^^^^^^^^^^^ unresolved import

use {does_not_exist::*, does_exist};
   //^^^^^^^^^^^^^^^^^ unresolved import

use does_not_exist::{
    a,
  //^ unresolved import
    b,
  //^ unresolved import
    c,
  //^ unresolved import
};

mod does_exist {
    pub struct Exists;
}
"#,
        );
    }

    #[test]
    fn dedup_unresolved_import_from_unresolved_crate() {
        check_diagnostics(
            r#"
//- /main.rs crate:main
mod a {
    extern crate doesnotexist;
  //^^^^^^^^^^^^^^^^^^^^^^^^^^ unresolved extern crate

    // Should not error, since we already errored for the missing crate.
    use doesnotexist::{self, bla, *};

    use crate::doesnotexist;
      //^^^^^^^^^^^^^^^^^^^ unresolved import
}

mod m {
    use super::doesnotexist;
      //^^^^^^^^^^^^^^^^^^^ unresolved import
}
"#,
        );
    }
}