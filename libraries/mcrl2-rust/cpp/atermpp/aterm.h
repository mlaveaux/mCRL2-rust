#pragma once
#include <memory>
#include <string>
#include <vector>
#include <iostream>

#include "rust/cxx.h"

#include "mcrl2/atermpp/aterm.h"
#include "mcrl2/atermpp/aterm_io_text.h"
#include "mcrl2/atermpp/detail/aterm_hash.h"

#include "mcrl2/data/application.h"
#include "mcrl2/data/data_expression.h"
#include "mcrl2/data/parse.h"
#include "mcrl2/data/function_symbol.h"

#include "mcrl2-rust/src/atermpp.rs.h"

using namespace mcrl2::data;

namespace atermpp
{

inline void initialise()
{
  // Enable debugging messages.
  mcrl2::log::mcrl2_logger::set_reporting_level(mcrl2::log::debug);
}

inline std::unique_ptr<aterm> new_aterm()
{
  return std::make_unique<aterm>();
}

std::unique_ptr<aterm> create_aterm(const function_symbol& symbol, rust::Slice<const aterm_ref> arguments)
{
  // TODO: This is some truly horrendous code that must be removed asap.
  std::vector<unprotected_aterm> converted_arguments;
  for (const aterm_ref& argument : arguments)
  {
    converted_arguments.push_back(unprotected_aterm(reinterpret_cast<detail::_aterm*>(argument.index)));
  }
  
  return std::make_unique<aterm>(aterm_appl(symbol, converted_arguments.begin(), converted_arguments.end()));
}

std::unique_ptr<aterm> aterm_from_string(rust::String text)
{
  return std::make_unique<aterm>(atermpp::read_term_from_string(static_cast<std::string>(text)));
}

std::size_t aterm_pointer(const aterm& term)
{
  return reinterpret_cast<std::size_t>(detail::address(term));
}

rust::String print_aterm(const aterm& term)
{
  std::stringstream str;
  str << term;
  return str.str();
}

std::size_t hash_aterm(const aterm& term)
{
  std::hash<aterm> hash;
  return hash(term);
}

bool equal_aterm(const aterm& first, const aterm& second)
{
  return first == second;
}

bool less_aterm(const aterm& first, const aterm& second)
{
  return first < second;
}

std::unique_ptr<aterm> copy_aterm(const aterm& term)
{
  aterm result(term);
  return std::make_unique<aterm>(result);
}

std::unique_ptr<function_symbol> get_aterm_function_symbol(const aterm& term)
{
  return std::make_unique<function_symbol>(term.function());
}

rust::Str get_function_symbol_name(const function_symbol& symbol)
{
  return symbol.name();
}

std::size_t get_function_symbol_arity(const function_symbol& symbol)
{
  return symbol.arity();
}

std::size_t hash_function_symbol(const function_symbol& symbol)
{
  std::hash<function_symbol> hasher;
  return hasher(symbol);
}

bool less_function_symbols(const function_symbol& first, const function_symbol& second)
{
  return first < second;
}

bool equal_function_symbols(const function_symbol& first, const function_symbol& second)
{
  return first == second;
}

std::unique_ptr<function_symbol> copy_function_symbol(const function_symbol& symbol)
{
  return std::make_unique<function_symbol>(symbol);
}

bool ffi_is_variable(const aterm& term)
{
  return mcrl2::data::is_variable(term);
}

std::unique_ptr<aterm> ffi_create_variable(rust::String name)
{
  return std::make_unique<aterm>(mcrl2::data::variable(static_cast<std::string>(name), mcrl2::data::sort_expression()));
}

std::unique_ptr<aterm> get_term_argument(const aterm& term, std::size_t index)
{
  return std::make_unique<aterm>(static_cast<const aterm_appl&>(term)[index]);
}

std::unique_ptr<function_symbol> create_function_symbol(rust::String name, std::size_t arity)
{
  return std::make_unique<function_symbol>(static_cast<std::string>(name), arity);
}

bool ffi_is_data_function_symbol(const aterm& term)
{
  return mcrl2::data::is_function_symbol(static_cast<const aterm_appl&>(term));
}

std::size_t ffi_get_function_symbol_index(const aterm& term)
{
  return atermpp::detail::index_traits<mcrl2::data::function_symbol,function_symbol_key_type, 2>::index(static_cast<const mcrl2::data::function_symbol&>(term));
}

std::size_t function_symbol_address(const function_symbol& symbol)
{
  return reinterpret_cast<std::size_t>(&symbol.name());
}

bool ffi_is_application(const aterm& term)
{
  return mcrl2::data::is_application(static_cast<const aterm_appl&>(term));
}

std::unique_ptr<aterm> ffi_create_application(const aterm& head, rust::Slice<const aterm_ref> arguments)
{
  // TODO: This is some truly horrendous code that must be removed asap.
  std::vector<data_expression> converted_arguments;
  for (const aterm_ref& argument : arguments)
  {
    converted_arguments.push_back(data_expression(aterm(reinterpret_cast<detail::_aterm*>(argument.index))));
  }

  return std::make_unique<mcrl2::data::application>(static_cast<const data_expression&>(head), converted_arguments.begin(), converted_arguments.end());
}

std::unique_ptr<aterm> ffi_create_data_function_symbol(rust::String name)
{
  return std::make_unique<mcrl2::data::function_symbol>(static_cast<std::string>(name), untyped_sort());
}


} // namespace atermpp