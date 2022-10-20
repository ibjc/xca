//core
pub mod math;

//periphery
pub mod wormhole;
pub mod account;
pub mod factory;
pub mod messages;
pub mod request;

//stargate
pub mod cosmos {
  pub mod base {
    pub mod query{
      pub mod v1beta1{
        include!("proto_shit/cosmos.base.query.v1beta1.rs");
      }
    }
    
    pub mod v1beta1{
      include!("proto_shit/cosmos.base.v1beta1.rs");
    }
  }
}

pub mod osmosis {

  pub mod gamm {
    pub mod v1beta1{
      include!("proto_shit/osmosis.gamm.v1beta1.rs");
    }

    pub mod poolmodels{
      pub mod balancer {
        pub mod v1beta1 {
          include!("proto_shit/osmosis.gamm.poolmodels.balancer.v1beta1.rs");
        }
      }
    }

  }

  pub mod lockup {
    include!("proto_shit/osmosis.lockup.rs");
  }

  pub mod superfluid {
    include!("proto_shit/osmosis.superfluid.rs");
  }

  pub mod tokenfactory{
    pub mod v1beta1{
      include!("proto_shit/osmosis.tokenfactory.v1beta1.rs");
    }
  }

}


