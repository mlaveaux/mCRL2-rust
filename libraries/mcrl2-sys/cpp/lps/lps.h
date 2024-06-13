#pragma once
#include <memory>
#include <string>

#include "rust/cxx.h"

#include "mcrl2/lps/io.h"
#include "mcrl2/utilities/exception.h"

namespace mcrl2::lps
{

std::unique_ptr<specification> read_linear_process_specification(rust::Str filename)
{
  auto result = std::make_unique<specification>();
  load_lps(*result, std::string(filename));
  return result;
}

std::unique_ptr<mcrl2::data::data_specification> get_data_specification(const specification& spec)
{
  return std::make_unique<mcrl2::data::data_specification>(spec.data());
}


rust::String print_linear_process_specification(const specification& spec)
{
  std::stringstream str;
  str << spec;
  return str.str();
}

} // namespace mcrl2::lps