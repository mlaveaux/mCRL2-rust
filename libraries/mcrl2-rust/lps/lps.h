#pragma once
#include <memory>
#include <string>

#include "rust/cxx.h"
#include "mcrl2/lps/detail/lps_io.h"
#include "mcrl2/utilities/exception.h"

namespace mcrl2::lps
{

std::unique_ptr<specification> read_linear_process_specification(rust::Str filename)
{
  specification result = detail::load_lps(std::string(filename));
  return std::make_unique<specification>(result);
}

rust::String print_linear_process_specification(const specification& spec)
{
  std::stringstream str;
  str << spec;
  return str.str();
}

} // namespace mcrl2::lps