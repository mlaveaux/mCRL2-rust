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

inline 
void initialise()
{
  // Enable debugging messages.
  mcrl2::log::logger::set_reporting_level(mcrl2::log::debug);
}

inline 
void collect_garbage()
{
  detail::g_thread_term_pool().collect();
}

inline
void print_metrics()
{
  detail::g_thread_term_pool().print_local_performance_statistics();
}

const detail::_aterm* aterm_address(const aterm& term)
{
  return detail::address(term);
}

const detail::_function_symbol* function_symbol_address(const function_symbol& symbol)
{
  return symbol.address();
}

std::unique_ptr<aterm> create_aterm(const detail::_function_symbol* symbol, rust::Slice<const detail::_aterm* const> arguments)
{
  rust::Slice<aterm> aterm_slice(const_cast<aterm*>(reinterpret_cast<const aterm*>(arguments.data())),
      arguments.length());

  return std::make_unique<aterm>(aterm_appl(function_symbol(symbol), aterm_slice.begin(), aterm_slice.end()));
}

std::unique_ptr<aterm> protect_aterm(const detail::_aterm* term)
{
  return std::make_unique<aterm>(term);
}

std::unique_ptr<function_symbol> protect_function_symbol(const detail::_function_symbol* symbol)
{
  return std::make_unique<function_symbol>(symbol);
}

std::unique_ptr<aterm> aterm_from_string(rust::String text)
{
  return std::make_unique<aterm>(atermpp::read_term_from_string(static_cast<std::string>(text)));
}

bool aterm_is_int(const detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  return t.type_is_int();
}

bool aterm_is_list(const detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  return t.type_is_list();
}

bool aterm_is_empty_list(const detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  return t.function() == detail::g_as_empty_list;
}

rust::String print_aterm(const detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  std::stringstream str;
  str << static_cast<const aterm&>(t);
  return str.str();
}

const detail::_function_symbol* get_aterm_function_symbol(const detail::_aterm* term)
{ 
  return atermpp::unprotected_aterm(term).function().address();
}

rust::Str get_function_symbol_name(const detail::_function_symbol* symbol)
{
  // TODO: This is not optimal.
  return function_symbol(symbol).name();
}

std::size_t get_function_symbol_arity(const detail::_function_symbol* symbol)
{
  return function_symbol(symbol).arity();
}

const detail::_aterm* get_term_argument(const detail::_aterm* term, std::size_t index)
{
  atermpp::unprotected_aterm t(term);
  return detail::address(static_cast<const aterm_appl&>(t)[index]);
}

std::unique_ptr<function_symbol> create_function_symbol(rust::String name, std::size_t arity)
{
  return std::make_unique<function_symbol>(static_cast<std::string>(name), arity);
}

// For the data namespace

bool is_data_function_symbol(const detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  return mcrl2::data::is_function_symbol(static_cast<const aterm_appl&>(t));
}

bool is_data_variable(const detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  return mcrl2::data::is_variable(static_cast<const aterm&>(t));
}

std::unique_ptr<aterm> create_data_variable(rust::String name)
{
  return std::make_unique<aterm>(mcrl2::data::variable(static_cast<std::string>(name), mcrl2::data::sort_expression()));
}

bool is_data_where_clause(const detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  return mcrl2::data::is_where_clause(static_cast<const aterm_appl&>(t));
}

bool is_data_abstraction(const detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  return mcrl2::data::is_abstraction(static_cast<const aterm_appl&>(t));
}

bool is_data_untyped_identifier(const detail::_aterm* term)
{
  atermpp::unprotected_aterm t(term);
  return mcrl2::data::is_untyped_identifier(static_cast<const aterm_appl&>(t));
}

std::unique_ptr<aterm> create_data_function_symbol(rust::String name)
{
  return std::make_unique<mcrl2::data::function_symbol>(static_cast<std::string>(name), untyped_sort());
}

std::unique_ptr<std::vector<aterm>> generate_types()
{
  return std::make_unique<std::vector<aterm>>();
}

} // namespace atermpp