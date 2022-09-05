#include "llvm/Pass.h"
#include "llvm/IR/Function.h"
#include "llvm/Support/raw_ostream.h"

#include "llvm/IR/LegacyPassManager.h"
#include "llvm/Transforms/IPO/PassManagerBuilder.h"

using namespace llvm;

namespace {
  class SanitizeAddress : public FunctionPass {
  public:
    static char ID;
    SanitizeAddress() : FunctionPass(ID) {}

    bool runOnFunction(Function &F) override {
      F.addFnAttr(Attribute::SanitizeAddress);
      return true;
    }
  };
}

static void registerSanitizeAddressPass(const PassManagerBuilder &, legacy::PassManagerBase &PM) {
  PM.add(new SanitizeAddress());                                  
}

char SanitizeAddress::ID = 0;
static RegisterPass<SanitizeAddress> X("sanitizeaddress", 
                            "SanitizeAddress Pass");

static RegisterStandardPasses 
    RegisterParmesanLLVMPass(PassManagerBuilder::EP_CGSCCOptimizerLate,
                              registerSanitizeAddressPass);