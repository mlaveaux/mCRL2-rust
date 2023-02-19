#pragma once
#include <memory>
#include <string>

#include "rust/cxx.h"

#include "mcrl2/atermpp/aterm.h"

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


}