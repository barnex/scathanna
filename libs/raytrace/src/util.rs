/// For to hit records (pairs of intersection distance `t` and some payload `v`),
/// return the frontmost, if any.
pub fn frontmost<T, V>(a: Option<(T, V)>, b: Option<(T, V)>) -> Option<(T, V)>
where
	V: PartialOrd,
{
	match (a, b) {
		(None, None) => None,
		(None, Some(hit)) => Some(hit),
		(Some(hit), None) => Some(hit),
		(Some(a), Some(b)) => {
			if a.1 < b.1 {
				Some(a)
			} else {
				Some(b)
			}
		}
	}
}
