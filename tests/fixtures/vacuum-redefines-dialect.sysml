package VacuumRedefinesDialect {
  part def Engine {
    part cylinders : Cylinder[2..*];
  }
  part def FourCylinderEngine :> Engine {
    part redefines cylinders[4];
  }
  part brushSystem : BrushSystem {
    part mainBrush : MainBrush {
      port redefines rotationSpeedIn;
    }
    part motor1 : dcMotor {
      part tire1 redefines motorTire[1];
    }
  }
  part def DriveController {
    attribute redefines architecture = EeArchitecture::arm;
  }
}
