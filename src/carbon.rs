/// Ponto central de Equações focadas no impacto Ambiental (Carbono e Termodinâmica).

/// Injeta a base termal física para conversão do consumo energético atômico em Massa de Carbono Equivalente (gCO2).
pub fn calculate_emissions(energy_joules: f64, intensity: f64) -> f64 {
    // A fórmula da equivalência: 
    // 3.6 Milhões de Joules equivale a Exatos 1 Qilowatt/hora (kWh). 
    // Multiplicando-se pela intensidade (Densidade de gCO2 por kWh na energia daquele estado), obtêm-se as Gramas limpas.
    (energy_joules / 3.6e6) * intensity
}

/// Feature Especial/Geográfica Regional.
/// Aborda o equacionamento em volume e peso equivalente numa amostra física visual.
pub fn get_salinas_equivalence(total_joules: f64) -> String {
    // Utilizando os Princípios fundamentais da Energia Potencial Gravitacional -> E = m * g * h
    // Onde 'm' (1 litro de água = 1kg), 'g' (Aceleração padrão de gravidade = 9.8m/s²) e 'h' (1 metro linear de altura)
    // 1 * 9.8 * 1 resulta em ~10 Joules para movimentar/arremessar exatamente 1 Litro de volume cúbico contra a gravidade.
    let litros = total_joules / 10.0;
    
    // Concatena de volta para ser servido pela Binding do Python numa formatação legível.
    format!("Esta computação gerou energia suficiente para mover {:.2} litros de água nas marés de Salinópolis.", litros)
}
