 

use std::io::Error;


use figures::units::{
    Px,
    UPx
};
use figures::IntoSigned;


use cushy::{
    styles,
    styles::Dimension,
    styles::IntoComponentValue,
    widgets
};
use cushy::widget::MakeWidget;
use cushy::value::{
    Source,
    Dynamic,
    DynamicReader,
    Destination
};



pub struct WorldSelectUI {
    pub screen: cushy::window::VirtualWindow,
    pub create_world: cushy::value::Dynamic<bool>,
    pub load_world: cushy::value::Dynamic<String>,
    pub opt_world: cushy::value::Dynamic<String>,
    pub back_to_title: cushy::value::Dynamic<bool>,
}

impl WorldSelectUI {
    pub fn new(worlds: Vec<(crate::wctx::world_loader::WorldPreview, cushy::kludgine::Texture)>, config: &wgpu::SurfaceConfiguration, device: &wgpu::Device, queue: &wgpu::Queue) -> WorldSelectUI {

        let create_world = cushy::value::Dynamic::new(false);

        let mut create_button = widgets::Button::new( widgets::Label::<&str>::new("Create New World") );
        create_button = create_button.kind( widgets::button::ButtonKind::Solid );
        create_button = create_button.on_click({
            let create_world = create_world.clone();
            move |click| { create_world.set(true); }
        });

        let back_to_title = cushy::value::Dynamic::new(false);

        let mut back_button = widgets::Button::new( widgets::Label::<&str>::new("Back") );
        back_button = back_button.kind( widgets::button::ButtonKind::Solid );
        back_button = back_button.on_click({
            let back = back_to_title.clone();
            move |click| { back.set(true); }
        });

        let load_world = cushy::value::Dynamic::new( "".to_string() );
        let opt_world = cushy::value::Dynamic::new( "".to_string() );

        // list of world options
        let mut worlds_list = cushy::widget::WidgetList::new();

        worlds_list.push( create_button.with_styles(Self::make_buttonstyles()) );

        for wp in worlds {
            worlds_list.push( Self::make_world_selector( wp.0.name, wp.1, load_world.clone(), opt_world.clone() ) );
        }

        // scroll to contain the world list
        let scroll = widgets::Scroll::vertical( worlds_list.into_rows() );

        let mut out_list = cushy::widget::WidgetList::new();
        out_list.push(scroll);
        out_list.push( back_button.with_styles(Self::make_buttonstyles()) );

        // outermost container
        let outer = widgets::Container::new( out_list.into_rows() ).pad_by(
            styles::Edges {
                top: Dimension::Px( Px::new(54)),
                bottom: Dimension::Px( Px::new(54)),
                left: Dimension::Px( Px::new(54)),
                right: Dimension::Px( Px::new(54))
            }
        ).transparent();


        let mut builder = cushy::window::StandaloneWindowBuilder::new( outer.fit_vertically() ).transparent();
        builder = builder.size( figures::Size { width: config.width, height: config.height } );
        let mut screen = builder.finish_virtual(device, queue);

        Self {
            screen,
            create_world,
            load_world,
            opt_world,
            back_to_title,
        }
    }



    pub fn make_world_selector( name: String, kl_texture: cushy::kludgine::Texture, load_clone: cushy::value::Dynamic<String>, opt_clone: cushy::value::Dynamic<String> ) -> widgets::Container {

        let preview_image = widgets::Image::new( cushy::kludgine::AnyTexture::Texture(kl_texture) ).scaled(0.25);

        let label = widgets::Label::new( name.clone() );

        let mut buttonlist = cushy::widget::WidgetList::new();
        let mut buttonstyles = Self::make_buttonstyles();

        let mut play_button = widgets::Button::new( widgets::Label::<&str>::new("PLAY") );
        play_button = play_button.kind( widgets::button::ButtonKind::Solid );
        play_button = play_button.on_click({
            let nm = name.clone();
            move |click| { load_clone.set(nm.clone()) }
        });
        buttonlist.push( play_button.with_styles(buttonstyles.clone()).expand_weighted(5) );

        let mut options_button = widgets::Button::new( widgets::Label::<&str>::new("OPTIONS") );
        options_button = options_button.kind( widgets::button::ButtonKind::Solid );
        options_button = options_button.on_click({
            let nm = name.clone();
            move |click| { opt_clone.set(nm.clone()) }
        });
        buttonlist.push( options_button.with_styles(buttonstyles.clone()).expand_weighted(5) );

        let mut right_list = cushy::widget::WidgetList::new();
        right_list.push(label);
        right_list.push(buttonlist.into_columns());

        let mut full_list = cushy::widget::WidgetList::new();
        full_list.push(preview_image);
        full_list.push(right_list.into_rows());

        widgets::Container::new( full_list.into_columns() ).pad_by(
            styles::Edges {
                top: Dimension::Px( Px::new(24)),
                bottom: Dimension::Px( Px::new(24)),
                left: Dimension::Px( Px::new(24)),
                right: Dimension::Px( Px::new(24))
            }
        ).background_color(
            styles::Color::new(0,0,0,240)
        )
    }

    pub fn make_buttonstyles() -> styles::Styles {
        let mut buttonstyles = styles::Styles::new();

        buttonstyles.insert( &styles::components::CornerRadius, styles::CornerRadii{ top_left: figures::units::Px::new(0), top_right: figures::units::Px::new(0), bottom_left: figures::units::Px::new(0), bottom_right: figures::units::Px::new(0) } );
        buttonstyles.insert( &styles::components::OutlineColor, styles::Color::new(0,0,0,0) );
        buttonstyles.insert( &styles::components::HighlightColor, styles::Color::new(0,0,0,0) );

        buttonstyles.insert( &widgets::button::ButtonOutline, styles::Color::new(224,173,83,255) );
        buttonstyles.insert( &widgets::button::ButtonHoverOutline, styles::Color::new(245,204,25,255) );
        buttonstyles.insert( &widgets::button::ButtonDisabledOutline, styles::Color::new(87,73,70,255) );
        buttonstyles.insert( &widgets::button::ButtonActiveOutline, styles::Color::new(218,123,33,255) );

        buttonstyles.insert( &widgets::button::ButtonBackground, styles::Color::new(0,0,0,0) );
        buttonstyles.insert( &widgets::button::ButtonHoverBackground, styles::Color::new(0,0,0,0) );
        buttonstyles.insert( &widgets::button::ButtonDisabledBackground, styles::Color::new(0,0,0,0) );
        buttonstyles.insert( &widgets::button::ButtonActiveBackground, styles::Color::new(0,0,0,0) );

        buttonstyles.insert( &styles::components::FontFamily, styles::FamilyOwned::SansSerif );

        buttonstyles
    }

}



