#pragma once
#include <memory>

#include "rust/cxx.h"

#include "mcrl2/atermpp/aterm.h"

namespace atermpp
{

inline std::unique_ptr<aterm> new_aterm()
{
  return std::make_unique<aterm>();
}

}