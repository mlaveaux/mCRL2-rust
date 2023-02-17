#pragma once
#include <memory>

#include "rust/cxx.h"

namespace atermpp
{
class aterm
{

};

inline std::unique_ptr<aterm> new_aterm()
{
  return std::make_unique<aterm>();
}
}