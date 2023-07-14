#pragma once
#include <memory>

#include "rust/cxx.h"

#include "mcrl2/atermpp/aterm.h"
#include "mcrl2/data/parse.h"
#include "mcrl2/data/detail/rewrite/jitty.h"
//#include "mcrl2/data/detail/rewrite/jittyc.h"

namespace mcrl2::data
{

std::unique_ptr<data_specification> ffi_parse_data_specification(const rust::Str text)
{
    return std::make_unique<data_specification>(parse_data_specification(std::string(text)));
}

std::unique_ptr<atermpp::aterm> ffi_parse_data_expression(const rust::Str text, const data_specification& spec)
{
    return std::make_unique<atermpp::aterm>(static_cast<const atermpp::aterm&>(parse_data_expression(std::string(text), spec)));
}

std::unique_ptr<detail::RewriterJitty> ffi_create_jitty_rewriter(const data_specification& spec)
{
    used_data_equation_selector selector;
    return std::make_unique<detail::RewriterJitty>(detail::RewriterJitty(spec, selector));
}

/*#ifdef MCRL2_JITTYC_AVAILABLE
std::unique_ptr<detail::RewriterCompilingJitty> ffi_create_jittyc_rewriter(const data_specification& spec)
{
  
}
#endif // MCRL2_JITTYC_AVAILABLE*/

std::unique_ptr<atermpp::aterm> ffi_rewrite(detail::RewriterJitty& rewriter, const atermpp::aterm& term)
{
    detail::RewriterJitty::substitution_type subsitution;
    data_expression result = rewriter.rewrite(static_cast<const data_expression&>(term), subsitution);
    return std::make_unique<atermpp::aterm>(static_cast<const atermpp::aterm&>(result));
}

std::size_t ffi_get_data_function_symbol_index(const atermpp::aterm& term)
{
  return atermpp::detail::index_traits<mcrl2::data::function_symbol,function_symbol_key_type, 2>::index(static_cast<const mcrl2::data::function_symbol&>(term));
}

}