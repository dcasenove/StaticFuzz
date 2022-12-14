//===-- DiffConsumer.h - Difference Consumer --------------------*- C++ -*-===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//
//
// This header defines the interface to the LLVM difference Consumer
//
//===----------------------------------------------------------------------===//

#ifndef LLVM_TOOLS_LLVM_DIFF_DIFFCONSUMER_H
#define LLVM_TOOLS_LLVM_DIFF_DIFFCONSUMER_H

#include "DiffLog.h"
#include "llvm/ADT/DenseMap.h"
#include "llvm/ADT/SmallVector.h"
#include "llvm/IR/Value.h"
#include "llvm/Support/Casting.h"
#include "llvm/Support/raw_ostream.h"
#include "parmesan/IDAssigner.h"
#include <set>
#include <fstream>

namespace parmesan {};
namespace llvm {
class StringRef;
  class Module;
  class Value;
  class Function;

  /// The interface for consumers of difference data.
  class Consumer {
    virtual void anchor();
  public:
    /// Record that a local context has been entered.  Left and
    /// Right are IR "containers" of some sort which are being
    /// considered for structural equivalence: global variables,
    /// functions, blocks, instructions, etc.
    virtual void enterContext(Value *Left, Value *Right) = 0;

    /// Record that a local context has been exited.
    virtual void exitContext() = 0;

    /// Record a difference within the current context.
    virtual void log(StringRef Text) = 0;

    /// Record a formatted difference within the current context.
    virtual void logf(const LogBuilder &Log) = 0;

    /// Record a line-by-line instruction diff.
    virtual void logd(const DiffLogBuilder &Log) = 0;

  protected:
    virtual ~Consumer() {}
  };

  class DiffConsumer : public Consumer {
  private:
    struct DiffContext {
      DiffContext(Value *L, Value *R)
        : L(L), R(R), Differences(false), IsFunction(isa<Function>(L)) {}
      Value *L;
      Value *R;
      bool Differences;
      bool IsFunction;
      DenseMap<Value*,unsigned> LNumbering;
      DenseMap<Value*,unsigned> RNumbering;
    };

    raw_ostream &out;
    SmallVector<DiffContext, 5> contexts;
    bool Differences;
    unsigned Indent;

    void printValue(Value *V, bool isL);
    void header();
    void indent();

    const parmesan::IDAssigner *IdAssigner;
    std::set<parmesan::IDAssigner::IdentifierType> DiffIdSet;

  public:
    DiffConsumer()
      : out(errs()), Differences(false), Indent(0) {}

    bool hadDifferences() const;
    void enterContext(Value *L, Value *R) override;
    void exitContext() override;
    void log(StringRef text) override;
    void logf(const LogBuilder &Log) override;
    void logd(const DiffLogBuilder &Log) override;
    void setIdAssigner(const parmesan::IDAssigner *);
    void printStats();
    void printStatsJson(raw_ostream &, std::ifstream &);
  };
}

#endif
