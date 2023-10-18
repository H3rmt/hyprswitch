use crate::SortableClient;
use hyprland::shared::WorkspaceId;

#[cfg(test)]
mod tests {
    use crate::test::MockClient;
    use crate::{sort, IgnoreWorkspaces};
    use std::collections::BTreeMap;

    /// ```
    ///       1       3    5   6     8   10  11  12
    ///    +----------------------------------------+
    /// 1  |  +-------+                      +---+  |
    /// 2  |  |   1   |              +---+   | 5 |  |
    /// 3  |  |       |    +---+     | 3 |   |   |  |
    /// 4  |  +-------+    | 2 |     +---+   |   |  |
    /// 5  |               +---+     +---+   |   |  |
    /// 6  |                         | 4 |   |   |  |
    /// 7  |    +-------+            +---+   +---+  |
    /// 8  |    |   6   |         +----+            |
    /// 9  |    |       |         | 7  |            |
    /// 10 |    +-------+         +----+            |
    ///    +----------------------------------------+
    ///         2       4         7    9
    /// ```
    #[test]
    fn test_big() {
        let ve = vec![
            MockClient(1, 1, 2, 3, 0, "1".to_string()),
            MockClient(5, 3, 1, 2, 0, "2".to_string()),
            MockClient(8, 2, 2, 2, 0, "3".to_string()),
            MockClient(8, 5, 2, 2, 0, "4".to_string()),
            MockClient(11, 1, 1, 6, 0, "5".to_string()),
            MockClient(2, 6, 2, 4, 0, "6".to_string()),
            MockClient(7, 8, 2, 2, 0, "7".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7"];

        let ve = sort(ve, None);

        println!("{ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    /// ```
    ///    1      2  3      4
    /// 1  +------+  +------+
    /// 2  |  1   |  |  2   |
    /// 3  |      |  |      |
    /// 4  +------+  +------+
    /// 5  +------+  +------+
    /// 6  |  3   |  |  4   |
    /// 7  +------+  +------+
    ///    1      2  3      4
    /// ```
    #[test]
    fn test_simple_1() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 0, "1".to_string()),
            MockClient(3, 1, 1, 3, 0, "2".to_string()),
            MockClient(1, 5, 1, 2, 0, "3".to_string()),
            MockClient(3, 5, 1, 2, 0, "4".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4"];

        let ve = sort(ve, None);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    /// ```
    ///    1      2  3      5
    /// 1  +------+  +------+
    /// 2  |  1   |  |  2   |
    /// 3  |      |  |      |
    /// 4  +------+  +------+
    /// 5  +---------+  +---+
    /// 6  |    3    |  | 4 |
    /// 7  +---------+  +---+
    ///    1         3  4   5
    /// /// ```
    #[test]
    fn test_x_difference_1() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 0, "1".to_string()),
            MockClient(3, 1, 2, 3, 0, "2".to_string()),
            MockClient(1, 5, 2, 2, 0, "3".to_string()),
            MockClient(4, 5, 1, 2, 0, "4".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4"];

        let ve = sort(ve, None);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    /// ```
    ///    1     2  3       6
    /// 1  +-----+  +-------+
    /// 2  |  1  |  |   2   |
    /// 3  |     |  |       |
    /// 4  +-----+  +-------+
    /// 5  +---------+  +---+
    /// 6  |    3    |  | 4 |
    /// 7  +---------+  +---+
    ///    1         4  5   6
    /// ```
    #[test]
    fn test_x_difference_2() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 0, "1".to_string()),
            MockClient(3, 1, 3, 3, 0, "2".to_string()),
            MockClient(1, 5, 3, 2, 0, "3".to_string()),
            MockClient(5, 5, 1, 2, 0, "4".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4"];

        let ve = sort(ve, None);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    /// ```
    ///    1      2  3      4
    /// 1  +------+  +------+
    /// 2  |  1   |  |  2   |
    /// 3  |      |  +------+
    /// 4  +------+  +------+
    /// 5  +------+  |  3   |
    /// 6  |  4   |  |      |
    /// 7  +------+  +------+
    ///    1      2  3      4
    /// ```
    #[test]
    fn test_y_difference_1() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 0, "1".to_string()),
            MockClient(3, 1, 1, 2, 0, "2".to_string()),
            MockClient(3, 4, 1, 3, 0, "3".to_string()),
            MockClient(1, 5, 1, 2, 0, "4".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4"];

        let ve = sort(ve, None);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    /// ```
    ///    1      2  3      4
    /// 1  +------+  +------+
    /// 2  |  1   |  |  2   |
    /// 3  |      |  +------+
    /// 4  |      |  +------+
    /// 5  +------+  |      |
    /// 6  +------+  |  3   |
    /// 7  |  4   |  |      |
    /// 8  +------+  +------+
    ///    1      2  3      4
    /// ```
    #[test]
    fn test_y_difference_2() {
        let ve = vec![
            MockClient(1, 1, 1, 4, 0, "1".to_string()),
            MockClient(3, 1, 1, 2, 0, "2".to_string()),
            MockClient(3, 4, 1, 4, 0, "3".to_string()),
            MockClient(1, 6, 1, 2, 0, "4".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4"];

        let ve = sort(ve, None);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    /// ```
    ///    1   2  4  5  6
    /// 1  +----+ +-----+
    /// 2  | 1  | |  3  |
    /// 3  |   +-----+  |
    /// 4  +---|  2  |  |
    /// 5  +---|     |--+
    /// 6  | 4 +-----+
    /// 7  +----+
    ///    1    3    5  6
    /// ```
    #[test]
    fn test_float() {
        let ve = vec![
            MockClient(1, 1, 2, 3, 0, "1".to_string()),
            MockClient(2, 3, 3, 3, 0, "2".to_string()),
            MockClient(4, 1, 2, 4, 0, "3".to_string()),
            MockClient(1, 5, 2, 2, 0, "4".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4"];

        let ve = sort(ve, None);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    /// ```
    ///                   Monitor 1
    ///       Workspace 1           Workspace 2
    /// 1  +------+  +------+ | +------+  +------+
    /// 2  |  1   |  |  2   |   |  3   |  |  4   |
    /// 3  |      |  |      | | |      |  +------+
    /// 4  +------+  +------+   +------+  +------+
    /// 5  +------+  +------+ | +------+  |  5   |
    /// 6  |  6   |  |  7   |   |  8   |  |      |
    /// 7  +------+  +------+ | +------+  +------+
    ///    1      2  3      4   1      2  3      4
    /// ```
    #[test]
    fn test_multiple_workspace_horizontal_ignore_workspace() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 0, "1".to_string()),
            MockClient(3, 1, 1, 3, 0, "2".to_string()),
            MockClient(1, 1, 1, 3, 1, "3".to_string()),
            MockClient(3, 1, 1, 2, 1, "4".to_string()),
            MockClient(3, 4, 1, 3, 1, "5".to_string()),
            MockClient(1, 5, 1, 2, 0, "6".to_string()),
            MockClient(3, 5, 1, 2, 0, "7".to_string()),
            MockClient(1, 5, 1, 2, 1, "8".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7", "8"];

        let mut workspaces = BTreeMap::new();
        // monitor 0 has 10 x 7 display
        // workspace 0 and 1 are on monitor 0
        workspaces.insert(0, (10, 7, 0));
        workspaces.insert(1, (10, 7, 1));

        let ve = sort(ve, Some(IgnoreWorkspaces::new(workspaces, false)));

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    /// ```
    ///                   Monitor 1
    ///       Workspace 1           Workspace 2
    /// 1  +------+  +------+ | +------+  +------+
    /// 2  |  1   |  |  2   |   |  5   |  |  6   |
    /// 3  |      |  |      | | |      |  +------+
    /// 4  +------+  +------+   +------+  +------+
    /// 5  +------+  +------+ | +------+  |  7   |
    /// 6  |  3   |  |  4   |   |  8   |  |      |
    /// 7  +------+  +------+ | +------+  +------+
    ///    1      2  3      4   1      2  3      4
    /// ```
    #[test]
    fn test_multiple_workspace_horizontal() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 0, "1".to_string()),
            MockClient(3, 1, 1, 3, 0, "2".to_string()),
            MockClient(1, 5, 1, 2, 0, "3".to_string()),
            MockClient(3, 5, 1, 2, 0, "4".to_string()),
            MockClient(1, 1, 1, 3, 1, "5".to_string()),
            MockClient(3, 1, 1, 2, 1, "6".to_string()),
            MockClient(3, 4, 1, 3, 1, "7".to_string()),
            MockClient(1, 5, 1, 2, 1, "8".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7", "8"];

        let mut workspaces = BTreeMap::new();
        // monitor 0 has 10 x 7 display
        // workspace 0 and 1 are on monitor 0
        workspaces.insert(0, (10, 7, 0));
        workspaces.insert(1, (10, 7, 1));

        let ve = sort(ve, None);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    /// ```
    ///    1      2  3      4
    /// 1  +------+  +------+
    /// 2  |  1   |  |  2   |
    /// 3  |      |  |      |      Workspace 1
    /// 4  +------+  +------+      Monitor 1
    /// 5  +------+  +------+
    /// 6  |  3   |  |  4   |
    /// 7  +------+  +------+
    ///
    ///   ------------------
    ///
    /// 1  +------+  +------+
    /// 2  |  5   |  |  6   |
    /// 3  |      |  +------+      Workspace 2
    /// 4  +------+  +------+      Monitor 1
    /// 5  +------+  |      |
    /// 6  |  8   |  |  7   |
    /// 7  +------+  +------+
    /// ```
    #[test]
    fn test_multiple_workspace_vertical_ignore_workspace() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 0, "1".to_string()),
            MockClient(3, 1, 1, 3, 0, "2".to_string()),
            MockClient(1, 5, 1, 2, 0, "3".to_string()),
            MockClient(3, 5, 1, 2, 0, "4".to_string()),
            MockClient(1, 1, 1, 3, 1, "5".to_string()),
            MockClient(3, 1, 1, 2, 1, "6".to_string()),
            MockClient(3, 4, 1, 3, 1, "7".to_string()),
            MockClient(1, 5, 1, 2, 1, "8".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7", "8"];

        let mut workspaces = BTreeMap::new();
        // monitor 0 has 10 x 7 display
        // workspace 0 and 1 are on monitor 0
        workspaces.insert(0, (10, 7, 0));
        workspaces.insert(1, (10, 7, 1));

        let ve = sort(ve, Some(IgnoreWorkspaces::new(workspaces, true)));

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }

    /// ```
    ///    1      2  3      4
    /// 1  +------+  +------+
    /// 2  |  1   |  |  2   |
    /// 3  |      |  |      |      Workspace 1
    /// 4  +------+  +------+      Monitor 1
    /// 5  +------+  +------+
    /// 6  |  3   |  |  4   |
    /// 7  +------+  +------+
    ///
    ///   ------------------
    ///
    /// 1  +------+  +------+
    /// 2  |  5   |  |  6   |
    /// 3  |      |  +------+      Workspace 2
    /// 4  +------+  +------+      Monitor 1
    /// 5  +------+  |      |
    /// 6  |  8   |  |  7   |
    /// 7  +------+  +------+
    /// ```
    #[test]
    fn test_multiple_workspace_vertical() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 0, "1".to_string()),
            MockClient(3, 1, 1, 3, 0, "2".to_string()),
            MockClient(1, 5, 1, 2, 0, "3".to_string()),
            MockClient(3, 5, 1, 2, 0, "4".to_string()),
            MockClient(1, 1, 1, 3, 1, "5".to_string()),
            MockClient(3, 1, 1, 2, 1, "6".to_string()),
            MockClient(3, 4, 1, 3, 1, "7".to_string()),
            MockClient(1, 5, 1, 2, 1, "8".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7", "8"];

        let ve = sort(ve, None);

        println!("ve: {ve:?}");
        assert_eq!(
            ve.iter().map(|v| v.5.to_string()).collect::<String>(),
            ve2.iter().map(|a| a.to_string()).collect::<String>()
        );
    }
}

#[derive(Debug)]
struct MockClient(i16, i16, i16, i16, WorkspaceId, String);

impl SortableClient for MockClient {
    fn x(&self) -> i16 {
        self.0
    }
    fn y(&self) -> i16 {
        self.1
    }
    fn w(&self) -> i16 {
        self.2
    }
    fn h(&self) -> i16 {
        self.3
    }
    fn ws(&self) -> WorkspaceId {
        self.4
    }
    fn set_x(&mut self, x: i16) {
        self.0 = x;
    }
    fn set_y(&mut self, y: i16) {
        self.1 = y;
    }
}
