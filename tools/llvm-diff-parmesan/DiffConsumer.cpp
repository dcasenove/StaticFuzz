//===-- DiffConsumer.cpp - Difference Consumer ------------------*- C++ -*-===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//
//
// This files implements the LLVM difference Consumer
// Modified for ParmeSan
//
//===----------------------------------------------------------------------===//

#include "DiffConsumer.h"
#include "llvm/IR/Instructions.h"
#include "llvm/Support/Debug.h"
#include "llvm/Support/ErrorHandling.h"

using namespace llvm;

static void ComputeNumbering(Function *F, DenseMap<Value*,unsigned> &Numbering){
  unsigned IN = 0;

  // Arguments get the first numbers.
  for (Function::arg_iterator
         AI = F->arg_begin(), AE = F->arg_end(); AI != AE; ++AI)
    if (!AI->hasName())
      Numbering[&*AI] = IN++;

  // Walk the basic blocks in order.
  for (Function::iterator FI = F->begin(), FE = F->end(); FI != FE; ++FI) {
    if (!FI->hasName())
      Numbering[&*FI] = IN++;

    // Walk the instructions in order.
    for (BasicBlock::iterator BI = FI->begin(), BE = FI->end(); BI != BE; ++BI)
      // void instructions don't get numbers.
      if (!BI->hasName() && !BI->getType()->isVoidTy())
        Numbering[&*BI] = IN++;
  }

  assert(!Numbering.empty() && "asked for numbering but numbering was no-op");
}


void Consumer::anchor() { }

void DiffConsumer::printValue(Value *V, bool isL) {
  if (V->hasName()) {
    out << (isa<GlobalValue>(V) ? '@' : '%') << V->getName();
    return;
  }
  if (V->getType()->isVoidTy()) {
    if (auto *SI = dyn_cast<StoreInst>(V)) {
      out << "store to ";
      printValue(SI->getPointerOperand(), isL);
    } else if (auto *CI = dyn_cast<CallInst>(V)) {
      out << "call to ";
      printValue(CI->getCalledValue(), isL);
    } else if (auto *II = dyn_cast<InvokeInst>(V)) {
      out << "invoke to ";
      printValue(II->getCalledValue(), isL);
    } else {
      out << *V;
    }
    return;
  }
  if (isa<Constant>(V)) {
    out << *V;
    return;
  }

  unsigned N = contexts.size();
  while (N > 0) {
    --N;
    DiffContext &ctxt = contexts[N];
    if (!ctxt.IsFunction) continue;
    if (isL) {
      if (ctxt.LNumbering.empty())
        ComputeNumbering(cast<Function>(ctxt.L), ctxt.LNumbering);
      out << '%' << ctxt.LNumbering[V];
      return;
    } else {
      if (ctxt.RNumbering.empty())
        ComputeNumbering(cast<Function>(ctxt.R), ctxt.RNumbering);
      out << '%' << ctxt.RNumbering[V];
      return;
    }
  }

  out << "<anonymous>";
}

void DiffConsumer::header() {
  if (contexts.empty()) return;
  for (SmallVectorImpl<DiffContext>::iterator
         I = contexts.begin(), E = contexts.end(); I != E; ++I) {
    if (I->Differences) continue;
    if (isa<Function>(I->L)) {
      // Extra newline between functions.
      if (Differences) out << "\n\n";

      Function *L = cast<Function>(I->L);
      Function *R = cast<Function>(I->R);
      if (L->getName() != R->getName())
        out << "in function " << L->getName()
            << " / " << R->getName() << ":\n";
      else
        out << "in function " << L->getName() << ":\n";
    } else if (isa<BasicBlock>(I->L)) {
      BasicBlock *L = cast<BasicBlock>(I->L);
      BasicBlock *R = cast<BasicBlock>(I->R);

      const parmesan::IDAssigner::IdentifiersMap *IdMap = &IdAssigner->getIdentifiersMap();
      auto diffId = IdMap->lookup(L);
      DiffIdSet.insert(diffId);

      if (L->hasName() && R->hasName() && L->getName() == R->getName()) {
        out << "  in block %" << L->getName() << ":\n";
        out << "  in block %" << L->getName() << "(" << diffId << ")"<< ":\n";
      } else {
        out << "  in block ";
        printValue(L, true);
        out << " / ";
        printValue(R, false);
        out << " (" << diffId << ")";
        out << ":\n";
      }
    } else if (isa<Instruction>(I->L)) {
      out << "    in instruction ";
      printValue(I->L, true);
      out << " / ";
      printValue(I->R, false);
      out << ":\n";
    }

    I->Differences = true;
  }
}

void DiffConsumer::indent() {
  unsigned N = Indent;
  while (N--) out << ' ';
}

bool DiffConsumer::hadDifferences() const {
  return Differences;
}

void DiffConsumer::enterContext(Value *L, Value *R) {
  contexts.push_back(DiffContext(L, R));
  Indent += 2;
}

void DiffConsumer::exitContext() {
  Differences |= contexts.back().Differences;
  contexts.pop_back();
  Indent -= 2;
}

void DiffConsumer::log(StringRef text) {
  header();
  indent();
  out << text << '\n';
}

void DiffConsumer::logf(const LogBuilder &Log) {
  header();
  indent();

  unsigned arg = 0;

  StringRef format = Log.getFormat();
  while (true) {
    size_t percent = format.find('%');
    if (percent == StringRef::npos) {
      out << format;
      break;
    }
    assert(format[percent] == '%');

    if (percent > 0) out << format.substr(0, percent);

    switch (format[percent+1]) {
    case '%': out << '%'; break;
    case 'l': printValue(Log.getArgument(arg++), true); break;
    case 'r': printValue(Log.getArgument(arg++), false); break;
    default: llvm_unreachable("unknown format character");
    }

    format = format.substr(percent+2);
  }

  out << '\n';
  out << "Diff IDs: ";
  for (auto e: DiffIdSet) {
      out << e << ", ";
  }
  out << "\n\n";
}

void DiffConsumer::logd(const DiffLogBuilder &Log) {
  header();

  for (unsigned I = 0, E = Log.getNumLines(); I != E; ++I) {
    indent();
    switch (Log.getLineKind(I)) {
    case DC_match:
      out << "  ";
      Log.getLeft(I)->print(dbgs()); dbgs() << '\n';
      //printValue(Log.getLeft(I), true);
      break;
    case DC_left:
      out << "< ";
      Log.getLeft(I)->print(dbgs()); dbgs() << '\n';
      //printValue(Log.getLeft(I), true);
      break;
    case DC_right:
      out << "> ";
      Log.getRight(I)->print(dbgs()); dbgs() << '\n';
      //printValue(Log.getRight(I), false);
      break;
    }
    //out << "\n";
  }
    
  
}

void DiffConsumer::printStats() {
  // Print BasicBlock IDs
  out << '\n';
  out << "Diff BB IDs: ";
  for (auto e: DiffIdSet) {
      out << e << " ";
  }
  out << "\n";

  // Print Cmp IDs
  parmesan::IDAssigner::CmpsMap cmps_bb = IdAssigner->getCmpMap();
  std::set<parmesan::IDAssigner::CmpIdType> cmps;
  for (auto const& e: cmps_bb) {
      for (auto bb: e.second) {
        if (DiffIdSet.count(bb) > 0)
          cmps.insert(e.first);
      }
  }
  out << "Diff Cmp IDs: ";
  for (auto cmp: cmps) {
    out << cmp << " ";
  }
  out << '\n';

  // Print Call Site dominators
  parmesan::IDAssigner::CallSiteDominators CSD = IdAssigner->getCallSiteDominators();
  out << "Call Site Dominators:" << "\n";
  for (auto const& e: CSD) {
      out << e.first << ": ";
      for (auto bb: e.second) {
          out << bb << ",";
      }
      out << "\n";
  }


}

void DiffConsumer::printStatsJson(raw_ostream &out, std::ifstream &src) {
  // Print BasicBlock IDs
  out << "{\n" << "\"targets\": [";

  // Print Cmp IDs
  parmesan::IDAssigner::CmpsMap cmps_bb = IdAssigner->getCmpMap();
  std::set<parmesan::IDAssigner::CmpIdType> cmps;
  for (auto const& e: cmps_bb) {
      for (auto bb: e.second) {
        if (DiffIdSet.count(bb) > 0)
          cmps.insert(e.first);
      }
  }
  bool first = true;
  for (auto cmp: cmps) {
    if (first) {
        first = false;
    } else {
        out << ", ";
    }
    out << cmp;
  }
  out << "],\n";

  // http://www.cplusplus.com/forum/general/90827/#msg488255
  std::string line;
  while( std::getline(src, line) ) out << line;
  out << ",\n";

  out << "\"id_mapping\": {";
  first = true;
  auto e = IdAssigner->getBBCmpMap();
  auto same_bb = e.equal_range(0);
  for(auto i = e.begin(); i != e.end(); i = same_bb.second) {
      if (first) {
          first = false;
      } else {
        out << ", ";
      }
      out << "\"" << i->first << "\"" << ": [";
      same_bb = e.equal_range(i->first);
      bool first_inner = true;
      for (auto cmpid = same_bb.first; cmpid != same_bb.second; cmpid++) {
        if (first_inner)
            first_inner = false;
        else
            out << ", ";
          out << cmpid->second;
      }
      out << "]";
  }
  out << "},\n";

  // Print Call Site dominators
  parmesan::IDAssigner::CallSiteDominators CSD = IdAssigner->getCallSiteDominators();
  out << "\"callsite_dominators\":" << "{";
  first = true;
  for (auto const& e: CSD) {
      if (first) {
          first = false;
      } else {
        out << ", ";
      }
      out << "\"" << e.first << "\"" << ": [";
      bool first_inner = true;
      for (auto bb: e.second) {
          if (first_inner)
              first_inner = false;
          else
              out << ", ";
          out << bb;
      }
      out << "]";
  }
  out << "}\n";


  out << "}";

}

void DiffConsumer::setIdAssigner(const parmesan::IDAssigner *assigner) {
    IdAssigner = assigner;
}
