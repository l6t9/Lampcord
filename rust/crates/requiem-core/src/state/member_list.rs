use crate::models::id::marker;
use crate::models;
use crate::presence::CustomStatus;
use requiem_ui::widget::primitives::PresenceStatus;

use super::wire::{GroupId, ListItem, ListOpKind, MemberListUpdate};
use crate::models::GuildMember;

#[derive(Clone, Debug)]
pub enum MemberListRow {
    Group { id: GroupId, count: usize },
    Member(Box<ListedMember>),
}

#[derive(Clone, Debug)]
pub struct ListedMember {
    pub member: GuildMember,
    pub status: PresenceStatus,

    pub custom_status: Option<CustomStatus>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupSpan {
    pub id: GroupId,
    pub count: usize,
    pub index: usize,
}

#[derive(Clone, Debug, Default)]
pub struct MemberList {
    pub id: String,
    pub rows: Vec<Option<MemberListRow>>,
    pub groups: Vec<GroupSpan>,
    pub member_count: usize,
    pub online_count: usize,
}

fn parse_row(item: &ListItem) -> Option<MemberListRow> {
    if let Some(group) = &item.group {
        return Some(MemberListRow::Group {
            id: group.id,
            count: group.count as usize,
        });
    }
    let member = item.member.as_ref()?;
    let presence = member.presence.as_ref();
    let status = presence
        .map(|p| PresenceStatus::from_gateway(p.status.as_deref().unwrap_or_default()))
        .unwrap_or_default();

    let custom_status = presence
        .and_then(|p| CustomStatus::from_activities(&p.activities))
        .filter(|_| status != PresenceStatus::Offline);
    Some(MemberListRow::Member(Box::new(ListedMember {
        member: member.guild_member(),
        status,
        custom_status,
    })))
}

impl MemberList {
    pub fn apply(&mut self, update: &MemberListUpdate) {
        if let Some(n) = update.member_count {
            self.member_count = n as usize;
        }
        if let Some(n) = update.online_count {
            self.online_count = n as usize;
        }
        for op in &update.ops {
            match op.op {
                ListOpKind::Sync => {
                    let start = op.range_start();
                    for (i, item) in op.items.iter().enumerate() {
                        self.set(start + i, parse_row(item));
                    }
                }

                ListOpKind::Invalidate => {
                    let (start, end) = (
                        op.range_start(),
                        op.range
                            .as_ref()
                            .and_then(|r| r.get(1).copied())
                            .unwrap_or(0),
                    );
                    for i in start..=end {
                        match self.rows.get_mut(i) {
                            Some(slot @ Some(_)) => *slot = None,
                            _ => break,
                        }
                    }
                }

                ListOpKind::Insert => {
                    if let (Some(i), Some(row)) = (op.index, op.item.as_ref().and_then(parse_row)) {
                        let i = i.min(self.rows.len());
                        self.rows.insert(i, Some(row));
                    }
                }
                ListOpKind::Update => {
                    if let (Some(i), Some(row)) = (op.index, op.item.as_ref().and_then(parse_row)) {
                        self.set(i, Some(row));
                    }
                }

                ListOpKind::Delete => {
                    if let Some(i) = op
                        .index
                        .filter(|&i| matches!(self.rows.get(i), Some(Some(_))))
                    {
                        self.rows.remove(i);
                    }
                }
                ListOpKind::Other => {}
            }
        }
        self.set_groups(update);
    }

    fn set(&mut self, index: usize, row: Option<MemberListRow>) {
        if row.is_none() && index >= self.rows.len() {
            return;
        }
        if index >= self.rows.len() {
            self.rows.resize(index + 1, None);
        }
        self.rows[index] = row;
    }

    fn set_groups(&mut self, update: &MemberListUpdate) {
        if update.groups.is_empty() && !self.groups.is_empty() {
            return;
        }
        let mut count = 0;
        self.groups = update
            .groups
            .iter()
            .map(|group| {
                let span = GroupSpan {
                    id: group.id,
                    count: group.count as usize,
                    index: count,
                };
                count += group.count as usize + 1;
                span
            })
            .collect();
        if !update.groups.is_empty() {
            self.rows.resize(count, None);
        }
    }

    pub fn loaded_rows(&self) -> impl Iterator<Item = &MemberListRow> {
        self.rows.iter().flatten()
    }

    pub fn custom_status_of(
        &self,
        user_id: models::Id<marker::UserMarker>,
    ) -> Option<CustomStatus> {
        self.loaded_rows().find_map(|row| match row {
            MemberListRow::Member(m) if m.member.user.id == user_id => m.custom_status.clone(),
            _ => None,
        })
    }

    pub fn status_of(
        &self,
        user_id: models::Id<marker::UserMarker>,
    ) -> Option<PresenceStatus> {
        self.loaded_rows().find_map(|row| match row {
            MemberListRow::Member(m) if m.member.user.id == user_id => Some(m.status),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn apply(list: &mut MemberList, mut value: Value) {
        value["guild_id"] = json!("1");
        list.apply(&serde_json::from_str(&value.to_string()).unwrap());
    }

    fn member(id: &str, status: &str) -> Value {
        json!({ "member": { "user": { "id": id, "username": format!("u{id}"), "discriminator": "0" }, "roles": [], "presence": { "status": status } } })
    }

    fn ids(list: &MemberList) -> Vec<String> {
        list.rows
            .iter()
            .map(|r| match r {
                None => "_".to_string(),
                Some(MemberListRow::Group { id, count }) => format!("g:{id}:{count}"),
                Some(MemberListRow::Member(m)) => m.member.user.id.to_string(),
            })
            .collect()
    }

    #[test]
    fn hoisted_role_groups_arrive_as_numbers() {
        let mut list = MemberList::default();
        apply(
            &mut list,
            json!({ "groups": [{ "id": 849317798111215666_u64, "count": 1 }, { "id": "online", "count": 1 }, { "id": "offline", "count": 0 }], "ops": [{ "op": "SYNC", "range": [0, 99], "items": [
            { "group": { "id": 849317798111215666_u64, "count": 1 } }, member("1", "online"),
            { "group": { "id": "online", "count": 1 } }, member("2", "online"),
        ] }] }),
        );
        assert_eq!(
            ids(&list),
            ["g:849317798111215666:1", "1", "g:online:1", "2", "_"]
        );
        assert_eq!(list.groups.len(), 3);
    }

    #[test]
    fn sync_then_granular_ops() {
        let mut list = MemberList::default();
        apply(
            &mut list,
            json!({ "member_count": 2, "online_count": 1, "groups": [{ "id": "online", "count": 1 }, { "id": "offline", "count": 1 }], "ops": [{ "op": "SYNC", "range": [0, 99], "items": [
            { "group": { "id": "online", "count": 1 } }, member("1", "online"),
            { "group": { "id": "offline", "count": 1 } }, member("2", "offline"),
        ] }] }),
        );
        assert_eq!(ids(&list), ["g:online:1", "1", "g:offline:1", "2"]);
        assert_eq!(
            list.status_of(models::Id::new(1)),
            Some(PresenceStatus::Online)
        );

        apply(
            &mut list,
            json!({ "ops": [
            { "op": "DELETE", "index": 3 },
            { "op": "DELETE", "index": 2 },
            { "op": "INSERT", "index": 2, "item": member("2", "idle") },
            { "op": "UPDATE", "index": 0, "item": { "group": { "id": "online", "count": 2 } } },
        ], "groups": [{ "id": "online", "count": 2 }, { "id": "offline", "count": 0 }] }),
        );
        assert_eq!(
            ids(&list),
            ["g:online:2", "1", "2", "_"],
            "the offline header is a row the list has but has not loaded"
        );
        assert_eq!(
            list.status_of(models::Id::new(2)),
            Some(PresenceStatus::Idle)
        );
    }

    #[test]
    fn out_of_range_ops_are_ignored() {
        let mut list = MemberList::default();
        apply(
            &mut list,
            json!({ "ops": [{ "op": "DELETE", "index": 5 }] }),
        );
        assert!(list.rows.is_empty());
    }

    #[test]
    fn the_list_is_as_long_as_its_groups_say_and_unloaded_rows_are_holes() {
        let mut list = MemberList::default();

        let mut items = vec![json!({ "group": { "id": "online", "count": 250 } })];
        items.extend((1..100).map(|i| member(&i.to_string(), "online")));
        apply(
            &mut list,
            json!({ "member_count": 250, "groups": [{ "id": "online", "count": 250 }], "ops": [{ "op": "SYNC", "range": [0, 99], "items": items }] }),
        );
        assert_eq!(list.rows.len(), 251);
        assert!(
            list.rows[..100].iter().all(Option::is_some)
                && list.rows[100..].iter().all(Option::is_none)
        );
        assert_eq!(
            list.groups,
            [GroupSpan {
                id: GroupId::Online,
                count: 250,
                index: 0
            }]
        );
    }

    #[test]
    fn a_second_range_fills_its_own_rows_and_an_invalidate_drops_them_again() {
        let mut list = MemberList::default();
        let groups = json!([{ "id": "online", "count": 5 }]);
        apply(
            &mut list,
            json!({ "groups": groups, "ops": [{ "op": "SYNC", "range": [0, 1], "items": [{ "group": { "id": "online", "count": 5 } }, member("1", "online")] }] }),
        );
        apply(
            &mut list,
            json!({ "groups": groups, "ops": [{ "op": "SYNC", "range": [4, 5], "items": [member("4", "online"), member("5", "online")] }] }),
        );
        assert_eq!(ids(&list), ["g:online:5", "1", "_", "_", "4", "5"]);
        apply(
            &mut list,
            json!({ "groups": groups, "ops": [{ "op": "INVALIDATE", "range": [4, 5] }] }),
        );
        assert_eq!(ids(&list), ["g:online:5", "1", "_", "_", "_", "_"]);
    }

    #[test]
    fn deleting_a_row_that_was_never_loaded_removes_nothing() {
        let mut list = MemberList::default();
        let groups = json!([{ "id": "online", "count": 3 }]);
        apply(
            &mut list,
            json!({ "groups": groups, "ops": [{ "op": "SYNC", "range": [0, 1], "items": [{ "group": { "id": "online", "count": 3 } }, member("1", "online")] }] }),
        );
        apply(
            &mut list,
            json!({ "groups": groups, "ops": [{ "op": "DELETE", "index": 3 }] }),
        );
        assert_eq!(ids(&list), ["g:online:3", "1", "_", "_"]);
    }
}
