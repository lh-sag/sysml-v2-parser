package PortDirectedAttributeRedefines {
    port def BasePort {
        out attribute suctionPower :> ISQ::power;
    }
    port def DerivedPort :> BasePort {
        out attribute redefines suctionPower :> ISQ::power;
    }
}
