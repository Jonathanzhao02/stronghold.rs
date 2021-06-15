initSidebarItems({"derive":[["RequestPermissions","Implements the [`VariantPermission`] for struct/ unions with PermissionValue(1). For enums, it implements [`ToPermissionVariants`], which creates an according new enum Permission with Unit variants, and implements [`VariantPermission`] by assigning different [`PermissionValue`] for each variant. The permission value is the “index” in the enum as exponent for the power of 2, thus from top to bottom 1, 2, 4, 8…"]],"enum":[["FirewallRule","Configure the firewall."],["RequestDirection","The direction of a [`CommunicationRequest::RequestMsg`] that firewall receives."]],"struct":[["FirewallConfiguration",""],["FirewallPermission","The sum of allowed permissions. This is using the same concepts as e.g. permission values in Unix systems."],["PermissionValue","The permission value for request variants. It is a  bit that is set at a certain index, therefore the value is always a power of 2."]],"trait":[["ToPermissionVariants","Convert an element to implement permissions."],["VariantPermission","The permission value for the different variants of an enum. This allows permitting specific variants of an enum while prohibiting others. In structs or unions, it should default to PermissionValue(1)"]]});