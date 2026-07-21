#![feature(proc_macro_expand)]

use itertools::Itertools;
use std::{collections::HashSet, num::NonZero};
use unsynn::{
    Comma, CommaDelimitedVec, Error, IParse, LiteralCharacter, LiteralInteger, LiteralString,
    ParenthesisGroupContaining, ToTokenIter, TrailingDelimiter::Forbidden, unsynn,
};

unsynn! {
    struct NonZeroPositive(LiteralInteger);
    parse_with |this, tokens| {
        match this.0.value() {
            0 => Error::other(None, tokens, "Dimension must be nonzero".to_string()),
            1..=18_446_744_073_709_551_615 => Ok(this),
            18_446_744_073_709_551_616.. => Error::other(None, tokens, "Dimension must fit into usize".to_string()),
        }
    };
}

unsynn! {
    struct UniqueChars(CommaDelimitedVec<LiteralCharacter, Forbidden>);
    parse_with |this, tokens| {
        if this.0.iter().map(|c| c.value.clone().value()).all_unique() {
            Ok(this)
        } else {
            Error::other(None, tokens, "All chars must be unique".to_string())
        }
    };
}

unsynn! {
    struct PerswizzleArgs {
        type_: LiteralString,
        comma1: Comma,
        dimensions: NonZeroPositive,
        comma2: Comma,
        chars: UniqueChars,
    }
}

#[proc_macro]
pub fn impl_perswizzle_functions(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = proc_macro2::TokenStream::from(tokens)
        .to_token_iter()
        .parse::<PerswizzleArgs>()
        .unwrap();

    let type_ = args.type_.value().trim_matches('"');
    let dimensions: NonZero<usize> = NonZero::new(args.dimensions.0.value() as usize).unwrap();
    let chars = args
        .chars
        .0
        .iter()
        .map(|char_literal| char_literal.value.value())
        .collect::<HashSet<_>>();

    parswizzle(dimensions, &chars)
        .iter()
        .map(|function_identifier| {
            let fields = function_identifier
                .chars()
                .map(|c| {
                    chars
                        .iter()
                        .position(|c_candidate| c == *c_candidate)
                        .unwrap()
                })
                .map(|i| format!("self[{i}]"))
                .reduce(|current_args, next_arg| current_args + ", " + &next_arg)
                .unwrap();
            format!(
                "
                    pub fn {function_identifier}(&self) -> {type_} {{
                        (
                            {fields}
                        ).into()
                    }}
                "
            )
            // }
        })
        .reduce(|current_fns, next_fn| current_fns + &next_fn)
        .unwrap()
        .parse()
        .unwrap()
}

unsynn! {
    struct SwizzleArgs {
        types: ParenthesisGroupContaining<CommaDelimitedVec<LiteralString, Forbidden>>,
        comma: Comma,
        chars: UniqueChars
    }
}

#[proc_macro]
pub fn impl_swizzle_functions(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = proc_macro2::TokenStream::from(tokens)
        .to_token_iter()
        .parse::<SwizzleArgs>()
        .unwrap();

    let types: Vec<&str> = args.types.content.iter().map(|s| s.value.value()).collect();
    let chars: HashSet<char> = args
        .chars
        .0
        .iter()
        .map(|char_literal| char_literal.value.value())
        .collect();

    let mut result = proc_macro::TokenStream::new();

    result.extend(
        types
            .into_iter()
            .enumerate()
            .map(|(i, type_)| {
                format!(
                    "{type_}, {}, {}",
                    i + 1,
                    chars
                        .iter()
                        .map(|c| format!("'{c}'"))
                        .reduce(|c1, c2| c1 + ", " + &c2)
                        .unwrap()
                )
            })
            .map(|stri| stri.parse::<proc_macro::TokenStream>().unwrap())
            .map(impl_perswizzle_functions),
    );
    result
}

fn swizzle(dimensions: NonZero<usize>, chars: &HashSet<char>) -> HashSet<String> {
    (1..=dimensions.get())
        .flat_map(|dimension| parswizzle(NonZero::new(dimension).unwrap(), chars))
        .collect()
}

fn parswizzle(dimensions: NonZero<usize>, chars: &HashSet<char>) -> HashSet<String> {
    if dimensions.get() == 1 {
        return chars.iter().copied().map(|char| char.to_string()).collect();
    }

    chars
        .iter()
        .flat_map(|first_char| {
            parswizzle(
                NonZero::new(dimensions.get() - 1)
                    .expect("This should be caught by the recursion check thingy"),
                chars,
            )
            .iter()
            .map(|suffix_string| first_char.to_string() + suffix_string)
            .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parswizzle_xy_1d() {
        assert_eq!(
            parswizzle(NonZero::new(1).unwrap(), &HashSet::from(['x', 'y'])),
            ["x", "y"].map(std::string::ToString::to_string).into()
        );
    }

    #[test]
    fn parswizzle_xy_2d() {
        assert_eq!(
            parswizzle(NonZero::new(2).unwrap(), &HashSet::from(['x', 'y'])),
            ["xx", "xy", "yx", "yy"]
                .map(std::string::ToString::to_string)
                .into()
        );
    }
    #[test]
    fn parswizzle_xy_3d() {
        assert_eq!(
            parswizzle(NonZero::new(3).unwrap(), &HashSet::from(['x', 'y'])),
            ["xxx", "yxy", "yyy", "yxx", "yyx", "xyy", "xxy", "xyx"]
                .map(std::string::ToString::to_string)
                .into()
        );
    }
    #[test]
    fn parswizzle_xy_5d() {
        assert_eq!(
            parswizzle(NonZero::new(5).unwrap(), &HashSet::from(['x', 'y'])),
            [
                "yyyyy", "yyyyx", "yyyxy", "yyyxx", "yyxyy", "yyxyx", "yyxxy", "yyxxx", "yxyyy",
                "yxyyx", "yxyxy", "yxyxx", "yxxyy", "yxxyx", "yxxxy", "yxxxx", "xyyyy", "xyyyx",
                "xyyxy", "xyyxx", "xyxyy", "xyxyx", "xyxxy", "xyxxx", "xxyyy", "xxyyx", "xxyxy",
                "xxyxx", "xxxyy", "xxxyx", "xxxxy", "xxxxx"
            ]
            .map(std::string::ToString::to_string)
            .into()
        );
    }

    #[test]
    fn parswizzle_x_1d() {
        assert_eq!(
            parswizzle(NonZero::new(1).unwrap(), &HashSet::from(['x'])),
            ["x"].map(std::string::ToString::to_string).into()
        );
    }
    #[test]
    fn parswizzle_x_5d() {
        assert_eq!(
            parswizzle(NonZero::new(5).unwrap(), &HashSet::from(['x'])),
            ["xxxxx"].map(std::string::ToString::to_string).into()
        );
    }

    #[test]
    fn parswizzle_xyz_3d() {
        assert_eq!(
            parswizzle(NonZero::new(3).unwrap(), &HashSet::from(['x', 'y', 'z'])),
            [
                "zzz", "zzy", "zzx", "zyz", "zyy", "zyx", "zxz", "zxy", "zxx", "yzz", "yzy", "yzx",
                "yyz", "yyy", "yyx", "yxz", "yxy", "yxx", "xzz", "xzy", "xzx", "xyz", "xyy", "xyx",
                "xxz", "xxy", "xxx"
            ]
            .map(std::string::ToString::to_string)
            .into()
        );
    }

    #[test]
    fn parswizzle_xyz_5d() {
        assert_eq!(
            parswizzle(NonZero::new(5).unwrap(), &HashSet::from(['x', 'y', 'z'])),
            [
                "zzzzz", "zzzzy", "zzzzx", "zzzyz", "zzzyy", "zzzyx", "zzzxz", "zzzxy", "zzzxx",
                "zzyzz", "zzyzy", "zzyzx", "zzyyz", "zzyyy", "zzyyx", "zzyxz", "zzyxy", "zzyxx",
                "zzxzz", "zzxzy", "zzxzx", "zzxyz", "zzxyy", "zzxyx", "zzxxz", "zzxxy", "zzxxx",
                "zyzzz", "zyzzy", "zyzzx", "zyzyz", "zyzyy", "zyzyx", "zyzxz", "zyzxy", "zyzxx",
                "zyyzz", "zyyzy", "zyyzx", "zyyyz", "zyyyy", "zyyyx", "zyyxz", "zyyxy", "zyyxx",
                "zyxzz", "zyxzy", "zyxzx", "zyxyz", "zyxyy", "zyxyx", "zyxxz", "zyxxy", "zyxxx",
                "zxzzz", "zxzzy", "zxzzx", "zxzyz", "zxzyy", "zxzyx", "zxzxz", "zxzxy", "zxzxx",
                "zxyzz", "zxyzy", "zxyzx", "zxyyz", "zxyyy", "zxyyx", "zxyxz", "zxyxy", "zxyxx",
                "zxxzz", "zxxzy", "zxxzx", "zxxyz", "zxxyy", "zxxyx", "zxxxz", "zxxxy", "zxxxx",
                "yzzzz", "yzzzy", "yzzzx", "yzzyz", "yzzyy", "yzzyx", "yzzxz", "yzzxy", "yzzxx",
                "yzyzz", "yzyzy", "yzyzx", "yzyyz", "yzyyy", "yzyyx", "yzyxz", "yzyxy", "yzyxx",
                "yzxzz", "yzxzy", "yzxzx", "yzxyz", "yzxyy", "yzxyx", "yzxxz", "yzxxy", "yzxxx",
                "yyzzz", "yyzzy", "yyzzx", "yyzyz", "yyzyy", "yyzyx", "yyzxz", "yyzxy", "yyzxx",
                "yyyzz", "yyyzy", "yyyzx", "yyyyz", "yyyyy", "yyyyx", "yyyxz", "yyyxy", "yyyxx",
                "yyxzz", "yyxzy", "yyxzx", "yyxyz", "yyxyy", "yyxyx", "yyxxz", "yyxxy", "yyxxx",
                "yxzzz", "yxzzy", "yxzzx", "yxzyz", "yxzyy", "yxzyx", "yxzxz", "yxzxy", "yxzxx",
                "yxyzz", "yxyzy", "yxyzx", "yxyyz", "yxyyy", "yxyyx", "yxyxz", "yxyxy", "yxyxx",
                "yxxzz", "yxxzy", "yxxzx", "yxxyz", "yxxyy", "yxxyx", "yxxxz", "yxxxy", "yxxxx",
                "xzzzz", "xzzzy", "xzzzx", "xzzyz", "xzzyy", "xzzyx", "xzzxz", "xzzxy", "xzzxx",
                "xzyzz", "xzyzy", "xzyzx", "xzyyz", "xzyyy", "xzyyx", "xzyxz", "xzyxy", "xzyxx",
                "xzxzz", "xzxzy", "xzxzx", "xzxyz", "xzxyy", "xzxyx", "xzxxz", "xzxxy", "xzxxx",
                "xyzzz", "xyzzy", "xyzzx", "xyzyz", "xyzyy", "xyzyx", "xyzxz", "xyzxy", "xyzxx",
                "xyyzz", "xyyzy", "xyyzx", "xyyyz", "xyyyy", "xyyyx", "xyyxz", "xyyxy", "xyyxx",
                "xyxzz", "xyxzy", "xyxzx", "xyxyz", "xyxyy", "xyxyx", "xyxxz", "xyxxy", "xyxxx",
                "xxzzz", "xxzzy", "xxzzx", "xxzyz", "xxzyy", "xxzyx", "xxzxz", "xxzxy", "xxzxx",
                "xxyzz", "xxyzy", "xxyzx", "xxyyz", "xxyyy", "xxyyx", "xxyxz", "xxyxy", "xxyxx",
                "xxxzz", "xxxzy", "xxxzx", "xxxyz", "xxxyy", "xxxyx", "xxxxz", "xxxxy", "xxxxx"
            ]
            .map(std::string::ToString::to_string)
            .into()
        );
    }

    #[test]
    fn swizzle_xy_1d() {
        assert_eq!(
            swizzle(NonZero::new(1).unwrap(), &HashSet::from(['x', 'y'])),
            ["x", "y"].map(std::string::ToString::to_string).into()
        );
    }

    #[test]
    fn swizzle_xy_2d() {
        assert_eq!(
            swizzle(NonZero::new(2).unwrap(), &HashSet::from(['x', 'y'])),
            ["xy", "x", "y", "xx", "yx", "yy"]
                .map(std::string::ToString::to_string)
                .into()
        );
    }
    #[test]
    fn swizzle_xy_3d() {
        assert_eq!(
            swizzle(NonZero::new(3).unwrap(), &HashSet::from(['x', 'y'])),
            [
                "yyy", "yy", "yx", "yxx", "y", "yxy", "yyx", "xxy", "xxx", "xy", "xx", "xyy", "x",
                "xyx"
            ]
            .map(std::string::ToString::to_string)
            .into()
        );
    }
    #[test]
    fn swizzle_xy_5d() {
        assert_eq!(
            swizzle(NonZero::new(5).unwrap(), &HashSet::from(['x', 'y'])),
            [
                "yyxy", "yyxxy", "xxxxy", "yxy", "yyyx", "yyyyy", "xyyx", "yyy", "yyx", "xx",
                "yxyyx", "xxxxx", "yxxy", "xyxy", "xxyyx", "xyyxx", "yxyyy", "yxx", "xyyyy",
                "xxxyx", "xxyxx", "yyxyy", "x", "xxyx", "xxyy", "yx", "xyyxy", "xxxyy", "yyxxx",
                "yxxyy", "yy", "xyxyy", "xxxy", "xxyyy", "yyyyx", "xxxx", "xyxxx", "xyxyx", "xyy",
                "yxyx", "yyxx", "xyxxy", "xxy", "yyxyx", "yxxx", "xxyxy", "xyyy", "xyxx", "xxx",
                "xyyyx", "yxxxy", "yxxyx", "xy", "xyx", "yxxxx", "yxyy", "yyyxy", "y", "yxyxx",
                "yyyy", "yyyxx", "yxyxy"
            ]
            .map(std::string::ToString::to_string)
            .into()
        );
    }

    #[test]
    fn swizzle_x_1d() {
        assert_eq!(
            swizzle(NonZero::new(1).unwrap(), &HashSet::from(['x'])),
            ["x"].map(std::string::ToString::to_string).into()
        );
    }
    #[test]
    fn swizzle_x_5d() {
        assert_eq!(
            swizzle(NonZero::new(5).unwrap(), &HashSet::from(['x'])),
            ["xxxxx", "xxx", "xx", "xxxx", "x"]
                .map(std::string::ToString::to_string)
                .into()
        );
    }

    #[test]
    fn swizzle_xyz_3d() {
        assert_eq!(
            swizzle(NonZero::new(3).unwrap(), &HashSet::from(['x', 'y', 'z'])),
            [
                "yxy", "yyx", "yxz", "x", "zzx", "yyz", "zyz", "yx", "zy", "yzz", "yyy", "zzz",
                "xyx", "xy", "zxz", "zz", "zzy", "yz", "xxx", "xzx", "xyz", "xzy", "xzz", "xx",
                "zyx", "xxz", "y", "zxy", "xz", "yzy", "zx", "z", "zxx", "xxy", "yy", "xyy", "zyy",
                "yzx", "yxx"
            ]
            .map(std::string::ToString::to_string)
            .into()
        );
    }

    #[test]
    fn swizzle_xyz_5d() {
        assert_eq!(
            swizzle(NonZero::new(5).unwrap(), &HashSet::from(['x', 'y', 'z'])),
            [
                "xzz", "yzxxy", "yxyyz", "zyzzz", "zyzy", "yzzy", "yzyz", "xz", "zzzzx", "xxz",
                "xxzx", "xyxyz", "yyxxz", "zyxxy", "xyxxy", "xyyxz", "xxxzz", "zzzx", "zxxxy",
                "zzyy", "xy", "zxyyz", "xxxxy", "zzyxx", "yyxxy", "yxxyy", "yxzyz", "xyzzy",
                "xxzzx", "xyyzx", "zzxx", "yzzx", "xyzzz", "zxyyy", "xxyzz", "zzxyx", "yzzxz",
                "zzyyx", "zyyzx", "yxyzz", "xxzzz", "xxzxy", "zzzyz", "zzzzz", "xxzxz", "yyzxx",
                "zzzxz", "yyzx", "zyyzz", "zyz", "zxzyx", "xzxxz", "xxyzy", "xzxzz", "yxz",
                "xyzyy", "yzy", "xzyx", "yxzzy", "xxxzy", "zyxzx", "zzyxy", "yxzxz", "yxzx",
                "xzzz", "zyxx", "yxzyx", "xxxy", "xzzzy", "yzzzx", "zxx", "xyxzy", "xzxzy",
                "zzxyy", "zyx", "yxxx", "yxyz", "zyyxy", "xzyzz", "xxxyz", "zzyzz", "zxzz",
                "xzxyz", "yyzyz", "zxyxx", "yzyzx", "xyyy", "xxzzy", "yyzz", "yx", "yzxz", "yyxyx",
                "zzxzy", "xyzy", "yzyxy", "zxxxx", "xyzzx", "xyyxx", "zzzz", "yxzxx", "xyy",
                "zzxy", "xzyyz", "yzyxz", "zxxyx", "yyyxy", "xzyyy", "zzzxx", "yyyz", "yzyyz",
                "xxxx", "yzzxx", "zyyyy", "yyzzy", "yxxxy", "yyx", "zzxyz", "xxzyy", "xxzyx",
                "yyyzx", "zxxxz", "zzxxy", "yxy", "yyzzz", "z", "xzzy", "xyzz", "xzzyy", "zxyxy",
                "zx", "xyxzx", "xxxxx", "yz", "zxyy", "yzx", "xyyyx", "zyyzy", "yzxx", "zzyyz",
                "yyzzx", "xzyxy", "zzxzz", "zxzzx", "zyxxx", "xxyxz", "xyz", "xyxyy", "yxyzx",
                "zzz", "zxzyy", "zxzxz", "xzyyx", "yzxyy", "xzxxx", "yzzxy", "yyxyz", "yyyyx",
                "yyxzz", "yxyxz", "zz", "zxxx", "xyyz", "xzxxy", "xxxz", "zyzzx", "zzyzx", "yxzzx",
                "zyxzz", "xxyyz", "zyxyy", "yyzxy", "xyxx", "zxxz", "yxxyz", "zzzyy", "xzyxx",
                "zyzyy", "yxyyx", "yxzz", "zyyyz", "xzxy", "yyyxz", "xxzxx", "yyzxz", "yyxyy",
                "zxxyy", "zxxzx", "zxyzz", "yxyzy", "yzzyy", "yyxzx", "yzyzy", "yzyxx", "yzxyz",
                "zxzx", "xxxyx", "yzzzz", "yxxyx", "yxyxy", "yzxy", "xyxz", "yyyzy", "zzxxz",
                "yzzz", "zxzzz", "zyxxz", "yyxy", "yyxxx", "zzzxy", "zzyzy", "zxy", "xxyyy",
                "yzzyx", "xyzxy", "yyxx", "yzxxx", "zyzyx", "xzxzx", "yzyx", "yyyy", "xyzxx",
                "yyyxx", "yxyyy", "yzzyz", "yzxzz", "xyzx", "zxyyx", "zzxzx", "yzxzx", "yyyyy",
                "x", "zyyz", "zxzyz", "yzzzy", "xyyyy", "zyzz", "yyy", "zzzy", "xxzy", "zxz",
                "xzyy", "xyyzy", "zxzxy", "yxzxy", "xxy", "yzyzz", "xyyx", "zzzzy", "xzyzy",
                "xyxxx", "zzyx", "xyzyx", "yzyy", "zyzzy", "xyxy", "yxxz", "xzyz", "xzzxz",
                "yzxyx", "zxzzy", "zyxyx", "xxyz", "xxx", "yyz", "zxzxx", "zyyyx", "zyzxy",
                "xzzxx", "zxxyz", "xxyyx", "zzy", "yzyyy", "xyzxz", "yxzyy", "yxx", "zyyx",
                "zzyxz", "xxyxx", "yxxzz", "yyxz", "zxxzy", "zzyyy", "zyyxx", "xzzyz", "yzxzy",
                "yxxzy", "xxzyz", "xzzx", "xyxzz", "zxxy", "yxyy", "xx", "zy", "zxxzz", "yyyzz",
                "yxyx", "zxyz", "xxxyy", "zxyxz", "xzxz", "xzx", "xyxxz", "zyzxx", "xzxyy",
                "xzzxy", "yy", "xyyxy", "yyxzy", "yyzyy", "zzyz", "zyxz", "xzzyx", "xxyzx", "xxzz",
                "yzxxz", "zyxzy", "xxxzx", "y", "zzx", "yyzyx", "zyy", "xzyxz", "xyxyx", "zxyx",
                "zyyy", "xzzzx", "xxyx", "zxzy", "yxxxx", "xzxx", "xyzyz", "yzyyx", "yxzzz",
                "zyzx", "yxzy", "zyxyz", "xzzzz", "yyyyz", "xxyxy", "xzy", "zzxxx", "yxyxx",
                "zxyzx", "yzz", "xzxyx", "zzzyx", "xyx", "zxyzy", "yxxy", "xyyzz", "zzxz", "yyzy",
                "zyzxz", "xyyyz", "zyyxz", "yyyx", "zyxy", "yxxzx", "xzyzx", "zyzyz", "xxyy",
                "yxxxz", "xxxxz"
            ]
            .map(std::string::ToString::to_string)
            .into()
        );
    }
}
