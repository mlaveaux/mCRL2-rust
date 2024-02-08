#pragma once
#include <iostream>
#include <memory>
#include <string>
#include <vector>


#include "rust/cxx.h"

#include "mcrl2/core/identifier_string.h"

#include "mcrl2/atermpp/aterm.h"
#include "mcrl2/atermpp/aterm_io_text.h"
#include "mcrl2/atermpp/detail/aterm_hash.h"
#include "mcrl2/atermpp/detail/aterm_pool_storage_implementation.h"

#include "mcrl2/data/application.h"
#include "mcrl2/data/data_expression.h"
#include "mcrl2/data/function_symbol.h"
#include "mcrl2/data/parse.h"

using namespace mcrl2::core;
using namespace mcrl2::data;
using namespace mcrl2::utilities;

namespace atermpp
{
  
using void_callback = rust::Fn<void(term_mark_stack&)>;
using size_callback = rust::Fn<std::size_t()>;

struct callback_container: detail::_aterm_container
{
  /// \brief Ensure that all the terms in the containers.
  inline void mark(term_mark_stack& todo) const override
  {
    callback_mark(todo);
  }

  virtual inline std::size_t size() const override
  {
    return callback_size();
  }

  callback_container(void_callback callback_mark, size_callback callback_size)
    : callback_mark(callback_mark),
      callback_size(callback_size)
  {}

  void_callback callback_mark;
  size_callback callback_size;
  term_mark_stack todo;
};

// What the fuck is this. Leaks the inner type because unions are not destructed automatically.
template<typename T>
class Leaker
{
public:
  union { T m_val; char dummy; };
  template<typename... Args>
  Leaker(Args... inputArgs) : m_val(inputArgs...) {}
  ~Leaker() {  }
};

using tls_callback_container = Leaker<callback_container>;

inline 
void initialise()
{
  // Enable debugging messages.
  mcrl2::log::logger::set_reporting_level(mcrl2::log::debug);

  // Create one application to initialise the DataAppl array.
  mcrl2::data::application test;
}

inline
void enable_automatic_garbage_collection(bool enabled)
{
  detail::g_term_pool().enable_garbage_collection(enabled);
}

/// Returns the number of terms in the pool.
std::size_t aterm_pool_size() {
  return detail::g_term_pool().size();
}

/// Returns the capacity of the pool, for terms of all arities so this is slightly misleading.
std::size_t aterm_pool_capacity()
{
  return detail::g_term_pool().capacity();
}

inline 
void collect_garbage()
{
  detail::g_thread_term_pool().collect();
}


void test_garbage_collection()
{
  detail::g_thread_term_pool().test_garbage_collection();
}

inline
void lock_shared() 
{
  detail::g_thread_term_pool().mutex().lock_shared_impl();
}

bool unlock_shared() 
{
  detail::g_thread_term_pool().mutex().unlock_shared();
  return !detail::g_thread_term_pool().mutex().is_shared_locked();
}

inline
void lock_exclusive() 
{
  detail::g_thread_term_pool().mutex().lock_impl();
}

void unlock_exclusive() 
{
  detail::g_thread_term_pool().mutex().unlock_impl();
}

inline
void print_metrics()
{
  detail::g_thread_term_pool().print_local_performance_statistics();
}

inline
std::unique_ptr<tls_callback_container> register_mark_callback(void_callback callback_mark, size_callback callback_size)
{
  // Do not destroy the callback_container since it is used in a TLS and destruction order w.r.t. the g_thread_aterm_pool is not guaranteed.
  return std::make_unique<tls_callback_container>(callback_mark, callback_size);
}

inline
void unregister_mark_callback(tls_callback_container& container)
{
  container.m_val.todo.~term_mark_stack();
}

const detail::_aterm* aterm_address(const aterm& term)
{
  return detail::address(term);
}

const detail::_aterm* create_aterm(const detail::_function_symbol* symbol, rust::Slice<const detail::_aterm* const> arguments)
{
  rust::Slice<aterm> aterm_slice(const_cast<aterm*>(reinterpret_cast<const aterm*>(arguments.data())),
      arguments.length());

  unprotected_aterm result(nullptr);
  make_term_appl(reinterpret_cast<aterm&>(result), function_symbol(symbol), aterm_slice.begin(), aterm_slice.end());
  return detail::address(result);
}

void aterm_mark_address(const detail::_aterm* term, term_mark_stack& todo)
{
  mark_term(*term, todo);
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
  return symbol->name();
}

std::size_t get_function_symbol_arity(const detail::_function_symbol* symbol)
{
  return symbol->arity();
}

const detail::_aterm* get_term_argument(const detail::_aterm* term, std::size_t index)
{
  atermpp::unprotected_aterm t(term);
  return detail::address(static_cast<const aterm_appl&>(t)[index]);
}

void protect_function_symbol(const detail::_function_symbol* symbol)
{
  symbol->increment_reference_count();
}

void drop_function_symbol(const detail::_function_symbol* symbol)
{
  symbol->decrement_reference_count();
}

const detail::_function_symbol* function_symbol_address(const function_symbol& symbol)
{
  return symbol.address();
}

const detail::_function_symbol*  create_function_symbol(rust::String name, std::size_t arity)
{
  Leaker<function_symbol> leak = Leaker<function_symbol>(static_cast<std::string>(name), arity);
  return leak.m_val.address();
}

std::unique_ptr<std::vector<aterm>> generate_types()
{
  return std::make_unique<std::vector<aterm>>();
}

} // namespace atermpp