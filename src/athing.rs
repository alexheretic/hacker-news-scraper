use select::{
    node::Node,
    predicate::{Class, Name},
};

/// Wrapper for a `tr.athing` html node
///
/// Contains uri, title & rank data plus indicates `AThingLine2`.
#[derive(Debug, Copy, Clone)]
pub struct AThing<'a>(pub Node<'a>);

impl<'a> AThing<'a> {
    pub fn uri_and_title(&self) -> Option<(String, String)> {
        let storylink = self.0.find(Class("storylink")).next()?;
        Some((storylink.attr("href")?.into(), storylink.text()))
    }

    pub fn rank(&self) -> Option<usize> {
        self.0.find(Class("rank")).next()?.extract_number_prefix()
    }

    pub fn line2(&self) -> Option<AThingLine2<'a>> {
        std::iter::successors(self.0.next(), |n| n.next())
            .find(|n| n.name() == Some("tr"))
            .map(AThingLine2)
    }
}

/// Wrapper for a `tr` that follows a `tr.athing` html node
///
/// Contains author, points & comments data.AsRef
#[derive(Debug, Copy, Clone)]
pub struct AThingLine2<'a>(pub Node<'a>);

impl<'a> AThingLine2<'a> {
    pub fn author(&self) -> Option<String> {
        self.0.find(Class("hnuser")).next()?.text().into()
    }

    pub fn points(&self) -> Option<usize> {
        self.0.find(Class("score")).next()?.extract_number_prefix()
    }

    pub fn comments(&self) -> Option<usize> {
        self.0.find(Name("a")).last()?.extract_number_prefix()
    }
}

trait NodeExt {
    /// Returns the numeric beginning of the text node as a number
    fn extract_number_prefix(&self) -> Option<usize>;
}

impl NodeExt for Node<'_> {
    fn extract_number_prefix(&self) -> Option<usize> {
        let text = self.first_child()?.as_text()?;
        let end = text.char_indices().take_while(|(_, c)| c.is_numeric()).last()?.0;
        text[..=end].parse().ok()
    }
}

#[cfg(test)]
mod athing_test {
    use super::*;
    use select::document::Document;

    const ATHING_FRAGMENT: &str = r#"
        <table>
        <tr class='athing' id='20820036'>
          <td align="right" valign="top" class="title"><span class="rank">22.</span></td>
          <td valign="top" class="votelinks">
            <center><a id='up_20820036' href='vote?id=20820036&amp;how=up&amp;goto=news%3Fp%3D1'>
                <div class='votearrow' title='upvote'></div>
              </a></center>
          </td>
          <td class="title"><a href="http://hardmath123.github.io/ambigrams.html" class="storylink">Words that do Handstands</a><span class="sitebit comhead"> (<a href="from?site=hardmath123.github.io"><span class="sitestr">hardmath123.github.io</span></a>)</span></td>
        </tr>
        <tr>
          <td colspan="2"></td>
          <td class="subtext">
            <span class="score" id="score_20820036">82 points</span> by <a href="user?id=hardmath123" class="hnuser">hardmath123</a> <span class="age"><a href="item?id=20820036">1 hour ago</a></span> <span id="unv_20820036"></span> | <a href="hide?id=20820036&amp;goto=news%3Fp%3D1">hide</a>
            | <a href="item?id=20820036">14&nbsp;comments</a> </td>
        </tr>
        </table>"#;

    #[test]
    fn parse_uri_and_title() {
        let document = Document::from(ATHING_FRAGMENT);
        let athing = AThing(document.find(Class("athing")).next().unwrap());

        let (uri, title) = athing.uri_and_title().expect("failed");
        assert_eq!(&uri, "http://hardmath123.github.io/ambigrams.html");
        assert_eq!(&title, "Words that do Handstands");
    }

    #[test]
    fn parse_rank() {
        let document = Document::from(ATHING_FRAGMENT);
        let athing = AThing(document.find(Class("athing")).next().unwrap());

        assert_eq!(athing.rank(), Some(22));
    }

    #[test]
    fn parse_author() {
        let document = Document::from(ATHING_FRAGMENT);
        let athing = AThing(document.find(Class("athing")).next().unwrap());
        let line2 = athing.line2().unwrap();
        assert_eq!(line2.author(), Some("hardmath123".into()));
    }

    #[test]
    fn parse_points() {
        let document = Document::from(ATHING_FRAGMENT);
        let athing = AThing(document.find(Class("athing")).next().unwrap());
        let line2 = athing.line2().unwrap();
        assert_eq!(line2.points(), Some(82));
    }

    #[test]
    fn parse_comments() {
        let document = Document::from(ATHING_FRAGMENT);
        let athing = AThing(document.find(Class("athing")).next().unwrap());
        let line2 = athing.line2().unwrap();
        assert_eq!(line2.comments(), Some(14));
    }
}
