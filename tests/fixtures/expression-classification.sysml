package ExpressionClassificationDemo {
	view def FilteredView {
		filter @SysML::PartUsage or @SysML::PortUsage;
		filter not @SysML::ConnectionUsage;
		filter @Approval and Approval::approved;
		filter guardExpr;
	}

	state def GuardedState {
		transition t
			if guardExpr
			then idle;
	}
}
