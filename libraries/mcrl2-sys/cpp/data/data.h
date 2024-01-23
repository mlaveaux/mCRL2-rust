#pragma once
#include <memory>

#include "rust/cxx.h"

#include "mcrl2/atermpp/aterm.h"
#include "mcrl2/core/identifier_string.h"
#include "mcrl2/data/detail/rewrite/jitty.h"
#include "mcrl2/data/sort_expression.h"
#include "mcrl2/data/parse.h"

#ifdef MCRL2_ENABLE_JITTYC
#include "mcrl2/data/detail/rewrite/jittyc.h"
#else
namespace mcrl2::data::detail
{
class RewriterCompilingJitty
{};
} // namespace mcrl2::data::detail
#endif // MCRL2_ENABLE_JITTYC

using namespace mcrl2::core;

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

std::unique_ptr<atermpp::aterm> parse_variable(const rust::Str text, const data_specification& spec)
{
  return std::make_unique<atermpp::aterm>(
      static_cast<const atermpp::aterm&>(mcrl2::data::parse_variable(std::string(text), spec)));
}

std::unique_ptr<detail::RewriterJitty> create_jitty_rewriter(const data_specification& spec)
{
  return std::make_unique<detail::RewriterJitty>(detail::RewriterJitty(spec, data::used_data_equation_selector(spec)));
}

#ifdef MCRL2_ENABLE_JITTYC
std::unique_ptr<detail::RewriterCompilingJitty> create_jitty_compiling_rewriter(const data_specification& spec)
{
  used_data_equation_selector selector;
  return std::make_unique<detail::RewriterCompilingJitty>(detail::RewriterCompilingJitty(spec, selector));
}
#else
std::unique_ptr<detail::RewriterCompilingJitty> create_jitty_compiling_rewriter(const data_specification& spec)
{
  return std::make_unique<detail::RewriterCompilingJitty>();
}
#endif // MCRL2_ENABLE_JITTYC

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
  data::used_data_equation_selector selector(data_spec);

  std::vector<data_equation> result;
  for (auto& equation : data_spec.equations()) 
  {
    if (selector(equation))
    {
      result.emplace_back(equation);
    }
  }

  return std::make_unique<std::vector<atermpp::aterm>>(result.begin(), result.end());
}

std::unique_ptr<std::vector<atermpp::aterm>> get_data_specification_constructors(const data_specification& data_spec, const atermpp::detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  sort_expression sort(static_cast<const atermpp::aterm_appl&>(t));
  auto constructors = data_spec.constructors(sort);
  return std::make_unique<std::vector<atermpp::aterm>>(constructors.begin(), constructors.end());
}

bool is_data_where_clause(const atermpp::detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  return is_where_clause(static_cast<const atermpp::aterm_appl&>(t));
}

bool is_data_abstraction(const atermpp::detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  return is_abstraction(static_cast<const atermpp::aterm_appl&>(t));
}

bool is_data_untyped_identifier(const atermpp::detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  return is_untyped_identifier(static_cast<const atermpp::aterm_appl&>(t));
}


bool is_data_function_symbol(const atermpp::detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  return mcrl2::data::is_function_symbol(static_cast<const atermpp::aterm_appl&>(t));
}

const atermpp::detail::_aterm* create_data_function_symbol(rust::String name)
{
  atermpp::unprotected_aterm result(nullptr);
  make_function_symbol(reinterpret_cast<atermpp::aterm_appl&>(result), identifier_string(static_cast<std::string>(name)), untyped_sort());
  return atermpp::detail::address(result);
}

bool is_data_variable(const atermpp::detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  return mcrl2::data::is_variable(static_cast<const atermpp::aterm&>(t));
}

const atermpp::detail::_aterm* create_data_variable(rust::String name)
{
  atermpp::unprotected_aterm result(nullptr);
  make_variable(reinterpret_cast<atermpp::aterm_appl&>(result), identifier_string(static_cast<std::string>(name)), sort_expression());
  return atermpp::detail::address(result);
}

bool is_data_sort_expression(const atermpp::detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  return mcrl2::data::is_sort_expression(static_cast<const atermpp::aterm_appl&>(t));
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