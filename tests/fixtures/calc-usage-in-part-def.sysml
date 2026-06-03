package P {
  part def BrushMotor {
    attribute par_maximumRPM : Real;
    calc actualRPM : CalcActualRPM {
      in rotationLevel : Real;
      return outSpeed : Real;
    }
  }
  calc def CalcActualRPM {
    in rotationLevel : Real;
    return outSpeed : Real;
  }
}
