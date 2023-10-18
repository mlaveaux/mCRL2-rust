#pragma once
#include <iostream>
#include <memory>
#include <string>
#include <vector>


#include "rust/cxx.h"

#include "mcrl2/atermpp/aterm.h"
#include "mcrl2/atermpp/aterm_io_text.h"
#include "mcrl2/atermpp/detail/aterm_hash.h"

#include "mcrl2/data/application.h"
#include "mcrl2/data/data_expression.h"
#include "mcrl2/data/function_symbol.h"
#include "mcrl2/data/parse.h"


#include "mcrl2-sys/src/atermpp.rs.h"

using namespace mcrl2::data;

namespace atermpp
{

inline void initialise()
{
  // Enable debugging messages.
  mcrl2::log::logger::set_reporting_level(mcrl2::log::debug);
}

inline void collect_garbage()
{
  detail::g_thread_term_pool().collect();
}

inline std::unique_ptr<aterm> new_aterm()
{
  return std::make_unique<aterm>();
}

std::unique_ptr<aterm> create_aterm(const function_symbol& symbol, rust::Slice<const aterm_ref> arguments)
{
  // Since aterm_ref and aterm have the same layout they can be transmuted. And honestly who is going to prevent that
  // anyway.
  rust::Slice<aterm> aterm_slice(const_cast<aterm*>(reinterpret_cast<const aterm*>(arguments.data())),
      arguments.length());
  return std::make_unique<aterm>(aterm_appl(symbol, aterm_slice.begin(), aterm_slice.end()));
}

std::unique_ptr<aterm> protect_aterm(const aterm& term)
{
  return std::make_unique<aterm>(term);
}

std::unique_ptr<aterm> aterm_from_string(rust::String text)
{
  return std::make_unique<aterm>(atermpp::read_term_from_string(static_cast<std::string>(text)));
}

std::size_t aterm_pointer(const aterm& term)
{
  return reinterpret_cast<std::size_t>(detail::address(term));
}

bool aterm_is_int(const aterm& term)
{
  return term.type_is_int();
}

bool aterm_is_list(const aterm& term)
{
  return term.type_is_list();
}

bool aterm_is_empty_list(const aterm& term)
{
  return term.function() == detail::g_as_empty_list;
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

std::unique_ptr<aterm> get_term_argument(const aterm& term, std::size_t index)
{
  return std::make_unique<aterm>(static_cast<const aterm_appl&>(term)[index]);
}

std::unique_ptr<function_symbol> create_function_symbol(rust::String name, std::size_t arity)
{
  return std::make_unique<function_symbol>(static_cast<std::string>(name), arity);
}

// For the data namespace

bool is_data_function_symbol(const aterm& term)
{
  return mcrl2::data::is_function_symbol(static_cast<const aterm_appl&>(term));
}

bool is_data_variable(const aterm& term)
{
  return mcrl2::data::is_variable(term);
}

std::unique_ptr<aterm> create_data_variable(rust::String name)
{
  return std::make_unique<aterm>(mcrl2::data::variable(static_cast<std::string>(name), mcrl2::data::sort_expression()));
}

bool is_data_where_clause(const aterm& term)
{
  return mcrl2::data::is_where_clause(static_cast<const aterm_appl&>(term));
}

bool is_data_application(const aterm& term)
{
  return mcrl2::data::is_application(static_cast<const aterm_appl&>(term));
}

bool is_data_expression(const aterm& term)
{
  return mcrl2::data::is_data_expression(static_cast<const aterm_appl&>(term));
}

bool is_data_abstraction(const aterm& term)
{
  return mcrl2::data::is_abstraction(static_cast<const aterm_appl&>(term));
}

bool is_data_untyped_identifier(const aterm& term)
{
  return mcrl2::data::is_untyped_identifier(static_cast<const aterm_appl&>(term));
}

std::unique_ptr<aterm> create_data_application(const aterm& head, rust::Slice<const aterm_ref> arguments)
{
  rust::Slice<data_expression> aterm_slice(
      const_cast<data_expression*>(reinterpret_cast<const data_expression*>(arguments.data())),
      arguments.length());

  return std::make_unique<mcrl2::data::application>(static_cast<const data_expression&>(head),
      aterm_slice.begin(),
      aterm_slice.end());
}

std::unique_ptr<aterm> create_data_function_symbol(rust::String name)
{
  return std::make_unique<mcrl2::data::function_symbol>(static_cast<std::string>(name), untyped_sort());
}

std::size_t function_symbol_address(const function_symbol& symbol)
{
  return reinterpret_cast<std::size_t>(&symbol.name());
}

std::unique_ptr<std::vector<aterm>> generate_types()
{
  return std::make_unique<std::vector<aterm>>();
}

} // namespace atermpp