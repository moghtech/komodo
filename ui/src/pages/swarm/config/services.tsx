import SwarmServicesSection, {
  SwarmServicesSectionProps,
} from "@/components/swarm/services-section";
import { useRead } from "@/lib/hooks";

export interface SwarmConfigServicesSectionProps extends Omit<
  SwarmServicesSectionProps,
  "services"
> {
  config: string | undefined;
}

export default function SwarmConfigServicesSection({
  id,
  config,
  ...props
}: SwarmConfigServicesSectionProps) {
  const services =
    useRead("ListSwarmServices", { swarm: id }).data?.filter(
      (service) => config && service.Configs.includes(config),
    ) ?? [];
  return <SwarmServicesSection id={id} services={services} {...props} />;
}
