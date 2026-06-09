package P {
    port def DebrisPort {
        inout item debris {
            attribute vol :> ISQ::volume;
            attribute mass :> ISQ::mass;
        }
    }
}
