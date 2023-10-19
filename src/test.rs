use crate::SortableClient;
use hyprland::shared::WorkspaceId;

#[cfg(test)]
mod tests {
    use crate::svg::{create_svg_from_client};
    use crate::test::MockClient;
    use crate::sort;
    use std::collections::BTreeMap;
    use std::time::Instant;

    /// ```
    ///       1       3    5   6     8   10  11  12
    ///    +----------------------------------------+
    /// 1  |  +-------+                      +---+  |
    /// 2  |  |   1   |              +---+   | 4 |  |
    /// 3  |  |       |    +---+     | 3 |   |   |  |
    /// 4  |  +-------+    | 2 |     +---+   |   |  |
    /// 5  |               +---+     +---+   |   |  |
    /// 6  |                         | 5 |   |   |  |
    /// 7  |    +-------+            +---+   +---+  |
    /// 8  |    |   6   |         +----+            |
    /// 9  |    |       |         | 7  |            |
    /// 10 |    +-------+         +----+            |
    ///    +----------------------------------------+
    ///         2       4         7    9
    /// ```
    #[test]
    fn test_big_1() {
        let ve = vec![
            MockClient(1, 1, 2, 3, 0, "1".to_string()),
            MockClient(5, 3, 1, 2, 0, "2".to_string()),
            MockClient(8, 2, 2, 2, 0, "3".to_string()),
            MockClient(11, 1, 1, 6, 0, "4".to_string()),
            MockClient(8, 5, 2, 2, 0, "5".to_string()),
            MockClient(2, 7, 2, 4, 0, "6".to_string()),
            MockClient(7, 8, 2, 2, 0, "7".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7"];

        let start = Instant::now();
        let ve = sort(ve, None);

        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_big_1.svg");
    }

    /// ```
    ///       1       3    5   6     8   10  11  12
    ///    +----------------------------------------+
    /// 1  |  +-------+                      +---+  |
    /// 2  |  |   1   |              +---+   | 4 |  |
    /// 3  |  |       |    +---+     | 3 |   |   |  |
    /// 4  |  +-------+    | 2 |     +---+   |   |  |
    /// 5  |    +-------+  |   |     +---+   |   |  |
    /// 6  |    |       |  +---+     | 7 |   |   |  |
    /// 7  |    |   5   |            +---+   +---+  |
    /// 8  |    |       |         +----+            |
    /// 9  |    +-------+         | 6  |            |
    /// 10 |                      +----+            |
    /// 11 | +--+                        +-------+  |
    /// 12 | |8 |                        |   9   |  |
    /// 13 | +--+                        +-------+  |
    ///    +----------------------------------------+
    ///      0  2       4         7    9
    /// ```
    #[test]
    fn test_big_2() {
        let ve = vec![
            MockClient(1, 1, 2, 3, 0, "1".to_string()),
            MockClient(5, 3, 1, 3, 0, "2".to_string()),
            MockClient(8, 2, 2, 2, 0, "3".to_string()),
            MockClient(11, 1, 1, 6, 0, "4".to_string()),
            MockClient(2, 5, 2, 4, 0, "5".to_string()),
            MockClient(7, 8, 2, 2, 0, "6".to_string()),
            MockClient(8, 5, 2, 2, 0, "7".to_string()),
            MockClient(0, 11, 1, 2, 0, "8".to_string()),
            MockClient(10, 11, 2, 2, 0, "9".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7", "8" , "9"];

        let start = Instant::now();
        let ve = sort(ve, None);
        
        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_big_2.svg");
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

        let start = Instant::now();
        let ve = sort(ve, None);

        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_simple_1.svg");
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

        let start = Instant::now();
        let ve = sort(ve, None);

        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_x_difference_1.svg");
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

        let start = Instant::now();
        let ve = sort(ve, None);

        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_x_difference_2.svg");
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

        let start = Instant::now();
        let ve = sort(ve, None);

        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_y_difference_1.svg");
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

        let start = Instant::now();
        let ve = sort(ve, None);

        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_y_difference_2.svg");
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

        let start = Instant::now();
        let ve = sort(ve, None);

        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_float.svg");
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
            MockClient(1, 1, 1, 3, 1, "1".to_string()),
            MockClient(3, 1, 1, 3, 1, "2".to_string()),
            MockClient(1, 1, 1, 3, 2, "3".to_string()),
            MockClient(3, 1, 1, 2, 2, "4".to_string()),

            MockClient(3, 4, 1, 3, 2, "5".to_string()),
            MockClient(1, 5, 1, 2, 1, "6".to_string()),
            MockClient(3, 5, 1, 2, 1, "7".to_string()),
            MockClient(1, 5, 1, 2, 2, "8".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7", "8"];

        let mut workspaces = BTreeMap::new();
        // monitor 0 has 5 x 7 display
        // workspace 0 and 1 are on monitor 0
        workspaces.insert(1, (0, 0));
        workspaces.insert(2, (5, 0));

        let start = Instant::now();
        let ve = sort(ve, Some(workspaces));

        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_multiple_workspace_horizontal_ignore_workspace.svg");
    }

    /// ```
    ///                   Monitor 1
    ///       Workspace 1           Workspace 2
    /// 1  +------+  +------+ | +------+  +------+
    /// 2  |  1   |  |  2   | | |  5   |  |  6   |
    /// 3  |      |  |      | | |      |  +------+
    /// 4  +------+  +------+ | +------+  +------+
    /// 5  +------+  +------+ | +------+  |  7   |
    /// 6  |  3   |  |  4   | | |  8   |  |      |
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

        let start = Instant::now();
        let ve = sort(ve, None);

        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_multiple_workspace_horizontal.svg");
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
    ///    ------------------
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
            MockClient(1, 1, 1, 3, 1, "1".to_string()),
            MockClient(3, 1, 1, 3, 1, "2".to_string()),
            MockClient(1, 5, 1, 2, 1, "3".to_string()),
            MockClient(3, 5, 1, 2, 1, "4".to_string()),
            MockClient(1, 1, 1, 3, 2, "5".to_string()),
            MockClient(3, 1, 1, 2, 2, "6".to_string()),
            MockClient(3, 4, 1, 3, 2, "7".to_string()),
            MockClient(1, 5, 1, 2, 2, "8".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7", "8"];

        let mut workspaces = BTreeMap::new();
        // monitor 0 has 5 x 7 display
        // workspace 0 and 1 are on monitor 0
        workspaces.insert(1, (0, 0));
        workspaces.insert(2, (0, 8));

        let start = Instant::now();
        let ve = sort(ve, Some(workspaces));

        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_multiple_workspace_vertical_ignore_workspace.svg");
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
    ///    ------------------
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

        let start = Instant::now();
        let ve = sort(ve, None);

        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_multiple_workspace_vertical.svg");
    }

    /// ```
    ///                   Monitor 1                                   Monitor 2
    ///       Workspace 1           Workspace 2           Workspace 3           Workspace 4  
    /// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
    /// 2  |  1   |  |  2   | | |  3   |  |  4   |  |  |  6   |  |  7   | | |  8   |  |  9   |
    /// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
    /// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
    /// 5  +------+  +------+ | +------+  |  5   |  |  +---------+  +---+ | +------+  |  10  |
    /// 6  |  11  |  |  12  | | |  13  |  |      |  |  |   14    |  |15 | | |  16  |  |      |
    /// 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
    ///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7      8
    /// ```
    #[test]
    fn test_multiple_workspace_multiple_monitor_horizontal_ignore_workspace() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 1, "1".to_string()),
            MockClient(3, 1, 1, 3, 1, "2".to_string()),
            MockClient(1, 1, 1, 3, 2, "3".to_string()),
            MockClient(3, 1, 1, 2, 2, "4".to_string()),
            MockClient(3, 4, 1, 3, 2, "5".to_string()),
            MockClient(5, 1, 1, 3, 3, "6".to_string()),
            MockClient(7, 1, 2, 3, 3, "7".to_string()),
            MockClient(5, 1, 1, 3, 4, "8".to_string()),
            MockClient(7, 1, 1, 2, 4, "9".to_string()),
            MockClient(7, 4, 1, 3, 4, "10".to_string()),
            MockClient(1, 5, 1, 2, 1, "11".to_string()),
            MockClient(3, 5, 1, 2, 1, "12".to_string()),
            MockClient(1, 5, 1, 2, 2, "13".to_string()),
            MockClient(5, 5, 2, 2, 3, "14".to_string()),
            MockClient(8, 5, 1, 2, 3, "15".to_string()),
            MockClient(5, 5, 1, 2, 4, "16".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16"];

        let mut workspaces = BTreeMap::new();
        // monitor 0 has 4 x 7 display
        // workspace 1 and 2 are on monitor 0, workspace 3 an 4 are on monitor 1
        workspaces.insert(1, (0, 0));
        workspaces.insert(2, (5, 0));
        
        workspaces.insert(3, (10, 0));
        workspaces.insert(4, (15, 0));

        let start = Instant::now();
        let ve = sort(ve, Some(workspaces));

        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_multiple_workspace_multiple_monitor_horizontal_ignore_workspace.svg");
    }

    /// ```
    ///                   Monitor 1                                   Monitor 2
    ///       Workspace 1           Workspace 2           Workspace 3           Workspace 4   
    /// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
    /// 2  |  1   |  |  2   | | |  3   |  |  4   |  |  |  6   |  |  7   | | |  8   |  |  9   |
    /// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
    /// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
    /// 5  +------+  +------+ | +------+  |  5   |  |  +---------+  +---+ | +------+  |  10  |
    /// 6  |  11  |  |  12  | | |  13  |  |      |  |  |   14    |  |15 | | |  16  |  |      |
    /// 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
    ///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7      8
    /// ```
    #[test]
    fn test_multiple_workspace_multiple_monitor_horizontal() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 1, "1".to_string()),
            MockClient(3, 1, 1, 3, 1, "2".to_string()),
            MockClient(1, 5, 1, 2, 1, "3".to_string()),
            MockClient(3, 5, 1, 2, 1, "4".to_string()),

            MockClient(1, 1, 1, 3, 2, "5".to_string()),
            MockClient(3, 1, 1, 2, 2, "6".to_string()),
            MockClient(3, 4, 1, 3, 2, "7".to_string()),
            MockClient(1, 5, 1, 2, 2, "8".to_string()),

            MockClient(5, 1, 1, 3, 3, "9".to_string()),
            MockClient(7, 1, 1, 3, 3, "10".to_string()),
            MockClient(5, 5, 1, 2, 3, "11".to_string()),
            MockClient(8, 5, 1, 2, 3, "12".to_string()),

            MockClient(5, 1, 1, 3, 4, "13".to_string()),
            MockClient(7, 1, 1, 2, 4, "14".to_string()),
            MockClient(7, 4, 1, 3, 4, "15".to_string()),
            MockClient(5, 5, 1, 2, 4, "16".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16"];

        let start = Instant::now();
        let ve = sort(ve, None);

        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_multiple_workspace_multiple_monitor_horizontal.svg");
    }


    /// ```
    ///                   Monitor 1                                   Monitor 2
    ///       Workspace 1           Workspace 2           Workspace 3           Workspace 4  
    /// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
    /// 2  |  1   |  |  2   | | |  5   |  |  6   |  |  |  9   |  |  10  | | |  13  |  |  14  |
    /// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
    /// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
    /// 5  +------+  +------+ | +------+  |  7   |  |  +---------+  +---+ | +------+  |  15  |
    /// 6  |  3   |  |  4   | | |  8   |  |      |  |  |   11    |  |12 | | |  16  |  |      |
    /// 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
    ///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7      8
    ///
    ///    -----------------------------------------------------------------------------------
    /// 
    ///                   Monitor 3                                   Monitor 4
    ///       Workspace 1           Workspace 2           Workspace 3           Workspace 4  
    /// 8  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
    /// 9  |  17  |  |  18  | | |  21  |  |  22  |  |  |  25  |  |  26  | | |  29  |  |  30  |
    /// 10 |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
    /// 11 +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
    /// 12 +------+  +------+ | +------+  |  23  |  |  +---------+  +---+ | +------+  |  32  |
    /// 13 |  19  |  |  20  | | |  24  |  |      |  |  |   27    |  |28 | | |  31  |  |      |
    /// 14 +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
    ///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7      8
    /// ```
    #[test]
    fn test_multiple_workspace_multiple_monitor_horizontal_vertical() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 1, "1".to_string()),
            MockClient(3, 1, 1, 3, 1, "2".to_string()),
            MockClient(1, 5, 1, 2, 1, "3".to_string()),
            MockClient(3, 5, 1, 2, 1, "4".to_string()),

            MockClient(1, 1, 1, 3, 2, "5".to_string()),
            MockClient(3, 1, 1, 2, 2, "6".to_string()),
            MockClient(3, 4, 1, 3, 2, "7".to_string()),
            MockClient(1, 5, 1, 2, 2, "8".to_string()),

            MockClient(5, 1, 1, 3, 3, "9".to_string()),
            MockClient(7, 1, 1, 3, 3, "10".to_string()),
            MockClient(5, 5, 1, 2, 3, "11".to_string()),
            MockClient(8, 5, 1, 2, 3, "12".to_string()),

            MockClient(5, 1, 1, 3, 4, "13".to_string()),
            MockClient(7, 1, 1, 2, 4, "14".to_string()),
            MockClient(7, 4, 1, 3, 4, "15".to_string()),
            MockClient(5, 5, 1, 2, 4, "16".to_string()),


            MockClient(1, 8, 1, 3, 5, "17".to_string()),
            MockClient(3, 8, 1, 3, 5, "18".to_string()),
            MockClient(1, 12, 1, 2, 5, "19".to_string()),
            MockClient(3, 12, 1, 2, 5, "20".to_string()),

            MockClient(1, 8, 1, 3, 6, "21".to_string()),
            MockClient(3, 8, 1, 2, 6, "22".to_string()),
            MockClient(3, 11, 1, 3, 6, "23".to_string()),
            MockClient(1, 12, 1, 2, 6, "24".to_string()),

            MockClient(5, 8, 1, 3, 7, "25".to_string()),
            MockClient(7, 8, 1, 3, 7, "26".to_string()),
            MockClient(5, 12, 1, 2, 7, "27".to_string()),
            MockClient(8, 12, 1, 2, 7, "28".to_string()),

            MockClient(5, 8, 1, 3, 8, "29".to_string()),
            MockClient(7, 8, 1, 2, 8, "30".to_string()),
            MockClient(7, 11, 1, 3, 8, "31".to_string()),
            MockClient(5, 12, 1, 2, 8, "32".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26","27", "28", "29", "30", "31", "32"];

        let start = Instant::now();
        let ve = sort(ve, None);

        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_multiple_workspace_multiple_monitor_horizontal_vertical.svg");
    }


    /// ```
    ///                   Monitor 1                                   Monitor 2
    ///       Workspace 1           Workspace 2           Workspace 3           Workspace 4   
    /// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
    /// 2  |  1   |  |  2   | | |  3   |  |  4   |  |  |  6   |  |  7   | | |  8   |  |  9   |
    /// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
    /// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
    /// 5  +------+  +------+ | +------+  |  5   |  |  +---------+  +---+ | +------+  |  10  |
    /// 6  |  11  |  |  12  | | |  13  |  |      |  |  |   14    |  |15 | | |  16  |  |      |
    /// 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
    ///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7      8
    ///
    ///    -----------------------------------------------------------------------------------
    /// 
    ///                   Monitor 3                                   Monitor 4
    ///       Workspace 5           Workspace 6           Workspace 7           Workspace 8   
    /// 1  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
    /// 2  |  17  |  |  18  | | |  19  |  |  20  |  |  |  22  |  |  23  | | |  24  |  |  25  |
    /// 3  |      |  |      | | |      |  +------+  |  |      |  |      | | |      |  +------+
    /// 4  +------+  +------+ | +------+  +------+  |  +------+  +------+ | +------+  +------+
    /// 5  +------+  +------+ | +------+  |  21  |  |  +---------+  +---+ | +------+  |  26  |
    /// 6  |  27  |  |  28  | | |  29  |  |      |  |  |   30    |  |31 | | |  32  |  |      |
    /// 7  +------+  +------+ | +------+  +------+  |  +---------+  +---+ | +------+  +------+
    ///    1      2  3      4   1      2  3      4     5      6  7  8   9   5      6  7      8
    /// ```
    #[test]
    fn test_multiple_workspace_multiple_monitor_horizontal_vertical_ignore_workspaces() {
        let ve = vec![
            MockClient(1, 1, 1, 3, 1, "1".to_string()),
            MockClient(3, 1, 1, 3, 1, "2".to_string()),

            MockClient(1, 1, 1, 3, 2, "3".to_string()),
            MockClient(3, 1, 1, 2, 2, "4".to_string()),
            MockClient(3, 4, 1, 3, 2, "5".to_string()),

            MockClient(5, 1, 1, 3, 3, "6".to_string()),
            MockClient(7, 1, 2, 3, 3, "7".to_string()),

            MockClient(5, 1, 1, 3, 4, "8".to_string()),
            MockClient(7, 1, 1, 2, 4, "9".to_string()),
            MockClient(7, 4, 1, 3, 4, "10".to_string()),

            MockClient(1, 5, 1, 2, 1, "11".to_string()),
            MockClient(3, 5, 1, 2, 1, "12".to_string()),

            MockClient(1, 5, 1, 2, 2, "13".to_string()),
            
            MockClient(5, 5, 2, 2, 3, "14".to_string()),
            MockClient(8, 5, 1, 2, 3, "15".to_string()),

            MockClient(5, 5, 1, 2, 4, "16".to_string()),


            MockClient(1, 8, 1, 3, 5, "17".to_string()),
            MockClient(3, 8, 1, 3, 5, "18".to_string()),

            MockClient(1, 8, 1, 3, 6, "19".to_string()),
            MockClient(3, 8, 1, 2, 6, "20".to_string()),
            MockClient(3, 11, 1, 3, 6, "21".to_string()),

            MockClient(5, 8, 1, 3, 7, "22".to_string()),
            MockClient(7, 8, 2, 3, 7, "23".to_string()),

            MockClient(5, 8, 1, 3, 8, "24".to_string()),
            MockClient(7, 8, 1, 2, 8, "25".to_string()),
            MockClient(7, 11, 1, 3, 8, "26".to_string()),

            MockClient(1, 12, 1, 2, 5, "27".to_string()),
            MockClient(3, 12, 1, 2, 5, "28".to_string()),

            MockClient(1, 12, 1, 2, 6, "29".to_string()),

            MockClient(5, 12, 2, 2, 7, "30".to_string()),
            MockClient(8, 12, 1, 2, 7, "31".to_string()),

            MockClient(5, 12, 1, 2, 8, "32".to_string()),
        ];
        let ve2 = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26","27", "28", "29", "30", "31", "32"];

        let mut workspaces = BTreeMap::new();
        // monitor 1, 2, 3 and 4 have 6 x 7 display
        // workspace 1 and 2 are on monitor 1, workspace 3 and 4 are on monitor 2, workspace 5 and 6 are on monitor 3, and workspace 7 an 8 are on monitor 4
        workspaces.insert(1, (0, 0));
        workspaces.insert(2, (5, 0));
        
        workspaces.insert(3, (10, 0));
        workspaces.insert(4, (15, 0));

        workspaces.insert(5, (0, 7));
        workspaces.insert(6, (5, 7));
        
        workspaces.insert(7, (10, 7));
        workspaces.insert(8, (15, 7));

        let start = Instant::now();
        let ve = sort(ve, Some(workspaces));

        println!("{ve:?} ({:?})", start.elapsed());
        assert_eq!(
            ve.iter().map(|v| v.5.to_string() + " ").collect::<String>(),
            ve2.iter().map(|a| a.to_string() + " ").collect::<String>()
        );

        create_svg_from_client(&ve, "svgs/test_multiple_workspace_multiple_monitor_horizontal_vertical_ignore_workspaces.svg");
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
    fn iden(&self) -> String {
        self.5.clone()
    }
}
