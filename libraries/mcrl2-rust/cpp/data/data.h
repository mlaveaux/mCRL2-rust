#pragma once
#include <memory>

#include "rust/cxx.h"

#include "mcrl2/data/parse.h"

namespace mcrl2::data
{

std::unique_ptr<data_specification> parse_data_specification(const rust::Str text)
{
  return std::make_unique<data_specification>(mcrl2::data::parse_data_specification(std::string(text)));
}

}