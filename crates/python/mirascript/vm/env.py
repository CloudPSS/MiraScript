from . import operations,helpers

# print(dir(operations))

vm_globals={k:getattr(operations,k) for k in dir(operations) if not k.startswith("_")}

for k in dir(helpers):
    if not k.startswith("_"):
        vm_globals[k]=getattr(helpers,k)
        
for k in dir(helpers):
    if not k.startswith("_"):
        vm_globals[k]=getattr(helpers,k)