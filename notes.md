todo:
	1. 	create error handling for when enough games aren't fetched
		maybe set timer from any1 from making *any* riot requests for 10 mins with a timer on frontend (?)

	2. 	determine how to compile a strength score given all stats above
		- each assist: 0.5,
		- each kill: 1.0,
		- each death: -1.0,
		- each 500 gold earned: 1.0,
		- each 1000 total damage dealt to champions: 1.0,
		- each 500 damage self-mitigated: 1.0,
		- each 1000 damage dealt to objectives: 1.0
		- each win: 5.0 (?)
		- each loss: -5.0 (?)