use ical::generator::IcalEvent;
use rustical_ical::{CalendarObject, CalendarObjectComponent, CalendarObjectType};

use crate::calendar::methods::report::calendar_query::{
    CompFilterElement, PropFilterElement, TimeRangeElement,
};

pub trait CompFilterable {
    fn get_comp_name(&self) -> &'static str;

    fn match_time_range(&self, time_range: &TimeRangeElement) -> bool;

    fn match_prop_filter(&self, prop_filter: &PropFilterElement) -> bool;

    fn match_subcomponents(&self, comp_filter: &CompFilterElement) -> bool;

    fn matches(&self, comp_filter: &CompFilterElement) -> bool {
        let name_matches = self.get_comp_name() != comp_filter.name;
        match (comp_filter.is_not_defined.is_some(), name_matches) {
            // We are the component that's not supposed to be defined
            (true, true) => return false,
            // We shall not be and indeed we aren't
            (true, false) => return true,
            // We don't match
            (false, false) => return false,
            _ => {}
        }

        if let Some(time_range) = comp_filter.time_range.as_ref()
            && !self.match_time_range(time_range)
        {
            return false;
        }

        for prop_filter in &comp_filter.prop_filter {
            if !self.match_prop_filter(prop_filter) {
                return false;
            }
        }

        // let subcomponents = self.get_subcomponents();
        // for sub_comp_filter in &comp_filter.comp_filter {
        // if sub_comp_filter.is_not_defined.is_some() {
        //     // If is_not_defined: Filter shuold match for all
        //     // Confusing logic but matching also means not being the component that
        //     // shouldn't be defined
        //     if subcomponents
        //         .iter()
        //         .any(|sub| !sub.matches(sub_comp_filter))
        //     {
        //         return false;
        //     }
        // } else {
        //     // otherwise if no component matches return false
        //     if !subcomponents.iter().any(|sub| sub.matches(sub_comp_filter)) {
        //         return false;
        //     }
        // }
        // }

        comp_filter
            .comp_filter
            .iter()
            .all(|filter| self.match_subcomponents(filter))
    }
}

impl CompFilterable for CalendarObject {
    fn get_comp_name(&self) -> &'static str {
        "VCALENDAR"
    }

    fn match_time_range(&self, _time_range: &TimeRangeElement) -> bool {
        false
    }

    fn match_prop_filter(&self, _prop_filter: &PropFilterElement) -> bool {
        // TODO
        true
    }

    fn match_subcomponents(&self, comp_filter: &CompFilterElement) -> bool {
        self.get_data().matches(comp_filter)
    }
}

impl CompFilterable for CalendarObjectComponent {
    fn get_comp_name(&self) -> &'static str {
        CalendarObjectType::from(self).as_str()
    }

    fn match_time_range(&self, time_range: &TimeRangeElement) -> bool {
        // TODO
        true
    }

    fn match_prop_filter(&self, _prop_filter: &PropFilterElement) -> bool {
        // TODO
        true
    }

    fn match_subcomponents(&self, _comp_filter: &CompFilterElement) -> bool {
        // TODO: Properly check subcomponents
        true
    }
}
