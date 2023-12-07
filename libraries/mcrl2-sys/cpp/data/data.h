#pragma once
#include <memory>

#include "rust/cxx.h"

#include "mcrl2/atermpp/aterm.h"
#include "mcrl2/data/detail/rewrite/jitty.h"
#include "mcrl2/data/parse.h"

#ifdef MCRL2_JITTYC_AVAILABLE
#include "mcrl2/data/detail/rewrite/jittyc.h"
#else
namespace mcrl2::data::detail
{
class RewriterCompilingJitty
{};
} // namespace mcrl2::data::detail
#endif // MCRL2_JITTYC_AVAILABLE

namespace mcrl2::data
{

std::unique_ptr<data_specification> parse_data_specification(const rust::Str text)
{
  return std::make_unique<data_specification>(parse_data_specification(std::string(text)));
}

std::unique_ptr<atermpp::aterm> parse_data_expression(const rust::Str text, const data_specification& spec)
{
  return std::make_unique<atermpp::aterm>(
      static_cast<const atermpp::aterm&>(parse_data_expression(std::string(text), spec)));
}

std::unique_ptr<detail::RewriterJitty> create_jitty_rewriter(const data_specification& spec)
{
  used_data_equation_selector selector;
  return std::make_unique<detail::RewriterJitty>(detail::RewriterJitty(spec, selector));
}

#ifdef MCRL2_JITTYC_AVAILABLE
std::unique_ptr<detail::RewriterCompilingJitty> ffi_create_jittyc_rewriter(const data_specification& spec)
{
  used_data_equation_selector selector;
  return std::make_unique<detail::RewriterCompilingJitty>(detail::RewriterCompilingJitty(spec, selector));
}
#endif // MCRL2_JITTYC_AVAILABLE

std::unique_ptr<atermpp::aterm> rewrite(detail::RewriterJitty& rewriter, const atermpp::detail::_aterm* term)
{
  detail::RewriterJitty::substitution_type subsitution;
  atermpp::unprotected_aterm t(term);

  data_expression result = rewriter.rewrite(static_cast<const data_expression&>(t), subsitution);
  return std::make_unique<atermpp::aterm>(static_cast<const atermpp::aterm&>(result));
}

std::size_t get_data_function_symbol_index(const atermpp::detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  return atermpp::detail::index_traits<mcrl2::data::function_symbol, function_symbol_key_type, 2>::index(
      static_cast<const mcrl2::data::function_symbol&>(t));
}

std::unique_ptr<data_specification> data_specification_clone(const data_specification& spec) 
{
  return std::make_unique<data_specification>(spec);  
}

std::unique_ptr<std::vector<atermpp::aterm>> get_data_specification_equations(const data_specification& data_spec)
{
  auto equations = data_spec.equations();
  return std::make_unique<std::vector<atermpp::aterm>>(equations.begin(), equations.end());
}

std::unique_ptr<atermpp::aterm> true_term() 
{
  return std::make_unique<atermpp::aterm>(data::sort_bool::true_());
}

std::unique_ptr<atermpp::aterm> false_term() 
{
  return std::make_unique<atermpp::aterm>(data::sort_bool::false_());
}

} // namespace mcrl2::data