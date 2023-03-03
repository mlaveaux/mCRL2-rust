#pragma once
#include <memory>
#include <string>

#include "rust/cxx.h"

#include "mcrl2/atermpp/aterm.h"
#include "mcrl2/atermpp/detail/aterm_hash.h"

#include "mcrl2/data/data_expression.h"

namespace atermpp
{

inline std::unique_ptr<aterm> new_aterm()
{
  return std::make_unique<aterm>();
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

std::unique_ptr<aterm> get_term_argument(const aterm& term, std::size_t index)
{
  return std::make_unique<aterm>(static_cast<const aterm_appl&>(term)[index]);
}

}